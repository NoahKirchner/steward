use crate::auth::*;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Request, Response,
};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::{
    default_on_request_failure, policies::ExponentialBackoff, RetryTransientMiddleware, Retryable,
    RetryableStrategy,
};
use serde_json::Value;
use std::{collections::HashMap, error::Error};

struct RetryStrategy;
impl RetryableStrategy for RetryStrategy {
    fn handle(&self, res: &reqwest_middleware::Result<reqwest::Response>) -> Option<Retryable> {
        match res {
            Ok(success) if success.status() != 200 => Some(Retryable::Transient),
            Ok(success) => None,
            Err(error) => default_on_request_failure(error),
        }
    }
}

pub async fn build_client() -> Result<StewardClient, Box<dyn Error>> {
    let auth_data = get_auth_variables()?;
    let mut headers = HeaderMap::new();
    let url = auth_data.address;
    let key = auth_data.key.as_str();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);

    let raw_client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;

    // This ClientBuilder uses the reqwest_middleware builder instead.
    let client = ClientBuilder::new(raw_client)
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            retry_policy,
            RetryStrategy,
        ))
        .build();

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
            let cluster_name = v["data"][0]["name"].to_string().replace("\"", "");
            Ok(StewardClient {
                client,
                url,
                headers,
                cluster_name,
                current_node: None,
            })
        }
        // Replace this god awfulness
        _ => {
            panic!("FIX ME PLERASE")
        }
    }
}

#[derive(Debug, Clone)]
pub struct StewardClient {
    client: reqwest_middleware::ClientWithMiddleware,
    url: String,
    headers: HeaderMap,
    pub cluster_name: String,
    pub current_node: Option<String>,
}

impl StewardClient {
    pub fn set_node(&mut self, node: String) -> Result<(), Box<dyn Error>> {
        self.current_node = Some(node);
        Ok(())
    }

    /*
     * This is sort of ugly, but basically when trying to get cluster/node information, proxmox
     * will return the "data" object that contains an array of objects. All this does is inspect
     * each object in the array, extract the name field from the object, and turn its data into a
     * hashmap, and then throws THAT into another hashmap (I know, it's horrible, forgive me) with
     * the key equal to the name of the node.
     */
    pub async fn about(&self) -> Result<HashMap<String, HashMap<String, Value>>, Box<dyn Error>> {
        let about = self
            .client
            .get(format!("{}/api2/json/cluster/status", self.url))
            //TODO Try to kill this .clone but it might not be possible
            .headers(self.headers.clone())
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
                node_map.insert(
                    node.remove("name").unwrap().as_str().unwrap().to_string(),
                    node,
                );
            }
        }

        Ok(node_map)
    }

    pub async fn clone_vm(
        &self,
        node: String,
        source_vmid: i32,
        clone_args: HashMap<&str, Value>,
    ) -> Result<(), Box<dyn Error>> {
        // TODO Check here to see if a pool exists or if a vmid is conflicting with the destination
        // otherwise the clone will fail
        //

        // REALLY BAD SOLUTION BUT we are on a time crunch my brutha
        let timeout = std::time::Duration::from_millis(1000);
        std::thread::sleep(timeout);
        let clone = self
            .client
            .post(format!(
                "{}/api2/json/nodes/{node}/qemu/{source_vmid}/clone",
                self.url
            ))
            .headers(self.headers.clone())
            .json(&clone_args)
            .send()
            .await?;
        //.text()
        //.await?;

        dbg!(clone);

        Ok(())
    }

    pub async fn destroy_vm(
        &self,
        node: String,
        vmid: i32,
        destroy_args: HashMap<&str, Value>,
    ) -> Result<(), Box<dyn Error>> {
        let destroy = self
            .client
            .delete(format!("{}/api2/json/nodes/{node}/qemu/{vmid}", self.url))
            .headers(self.headers.clone())
            // TODO figure out why sending arguments breaks vm destruction with 501 error
            //.json(&destroy_args)
            .send()
            .await?
            .text()
            .await?;

        dbg!(destroy);

        Ok(())
    }

    pub async fn vm_status(&self, node: String, vmid: i32) -> Result<(), Box<dyn Error>> {
        let status = self
            .client
            .get(format!(
                "{}/api2/json/nodes/{node}/qemu/{vmid}/status/current",
                self.url
            ))
            .headers(self.headers.clone())
            .send()
            .await?
            .text()
            .await?;

        dbg!(status);

        Ok(())
    }
}
