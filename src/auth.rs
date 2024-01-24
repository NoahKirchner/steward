use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde_json::{Value};
use std::collections::HashMap;
use std::error::Error;
use std::env::{self, set_var};

pub struct AuthData {
    pub address: String,
    pub key: String,
}

pub fn get_auth_variables() -> Result<AuthData, Box<dyn Error>> {
    let address = env::var("STEWARD_ADDRESS")?;
    let key = env::var("STEWARD_KEY")?;

    Ok(
        AuthData {
        address,
        key,
    }
    )
    
}

// TODO ADD FUNCTIONS FOR WRITING TO AND READING FROM CONFIG FILE


/* Pre API Key Authentication */
// TODO RESTRUCTURE THIS TO SAVE TO CONFIG FILE
pub async fn set_auth_variables(address:String, username:String, password:String) -> Result<(), Box<dyn Error>> {
    let addr = address.as_str();
    let user = username.as_str();
    let pass = password.as_str();

    let mut json_data = HashMap::new();
    json_data.insert("username", user);
    json_data.insert("password", pass);

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;

    let url = format!("{}/api2/json/access/ticket", addr);
    let raw_data = client
        .post(url)
        .json(&json_data)
        .send()
        .await?
        .text()
        .await?;
    
    // TODO FIX UNWRAPS

    let v: Value = serde_json::from_str(&raw_data)?;
    let csrf = v["data"]["CSRFPreventionToken"].clone();
    let ticket = format!("PVEAuthCookie={}", v["data"]["ticket"].clone().as_str().unwrap());

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(ticket.as_str())?);
    //TODO Remove this unwrap
    headers.insert("Csrfpreventiontoken", HeaderValue::from_str(csrf.as_str().unwrap())?,);

    let tokenid = "steward";
    let url = format!("{}/api2/json/access/users/{}/token/{}", addr, user, tokenid);
    let api_del = client
        .delete(url.clone())
        .headers(headers.clone())
        .send()
        .await?;

    let api_key = client 
        .post(url.clone())
        .headers(headers.clone())
        .send()
        .await?
        .text()
        .await?;

    let v: Value = serde_json::from_str(&api_key)?;
    // Remove unwrap PLEASE TODO PLEASE
    let uuid = v["data"]["value"].as_str().unwrap();
    let key = format!("PVEAPIToken={}!{}={}", user, tokenid, uuid);
    
    // TODO export these to a configuration file
    env::set_var("STEWARD_KEY", key);
    env::set_var("STEWARD_ADDRESS", addr);

    dbg!(get_auth_variables());
    

    Ok(())

}
