use crate::auth::*;
use std::{error::Error, collections::HashMap};
use reqwest::{ClientBuilder, header::{HeaderMap, AUTHORIZATION, HeaderValue}};
use serde_json::{Value};

pub async fn build_client()->Result<StewardClient, Box<dyn Error>>{
    let auth_data = get_auth_variables()?;
    let mut headers = HeaderMap::new();
    let url = auth_data.address;
    let key = auth_data.key.as_str();
    
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;

    headers.insert(AUTHORIZATION, HeaderValue::from_str(key)?);
    
    let config_url = format!("{}/api2/json/cluster/status", url.clone());
    let config = client 
        .get(config_url.clone())
        .headers(headers.clone())
        .send()
        .await?;

    match config.status().as_u16() {
        200 => {
            let config_data = config.text().await?;
            let v: Value = serde_json::from_str(&config_data)?;
            // I know this looks horrible, but it gets rid of the quotes around the cluster name.
            let cluster_name = v["data"][0]["name"].to_string();
            Ok(
                StewardClient {
                    client,
                    url,
                    headers,
                    cluster_name,
                }
                )
        }
        // Replace this god awfulness
        _ => { panic!("FIX ME PLERASE" )}
    }
}

#[derive(Debug, Clone)]
pub struct StewardClient {
    client: reqwest::Client,
    url: String,
    headers: HeaderMap,
    pub cluster_name: String,

}

impl StewardClient {

    pub async fn about(self)->Result<HashMap<String, HashMap<String, Value>>, Box<dyn Error>>{
        let about = self.client
            .get(format!("{}/api2/json/cluster/status", self.url))
            .headers(self.headers)
            .send()
            .await?
            .text()
            .await?;

        let v: Value = serde_json::from_str(about.as_str()).unwrap();
        let node_list: Vec<Value> = serde_json::from_value(v["data"].clone())?;


        // Lord forgive me for the sin of a nested hashmaps but structs would be even more stupid

        let mut node_map: HashMap<String, HashMap<String, Value>> = HashMap::new();
        for entry in node_list.into_iter() {
            let mut node: HashMap<String, Value> = serde_json::from_value(entry.clone()).unwrap();
            
            if node.get("name").is_some() {
                node_map.insert(node.remove("name").unwrap().as_str().unwrap().to_string(), node);
            }
        }

        Ok(node_map)
    }


}
