mod cli;
use cli::*;
mod auth;
use auth::*;
mod client;
use client::*;
use std::{process::exit, collections::HashMap};
use tokio;
use std::error::Error;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut rl = Repl::<Cli>::new();
    let mut client: Option<StewardClient> = Option::None;
    // Attempt to connect here if ya can my G TODO TODO TODO PLEASE TODO
    loop {
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command.command {
            ReplCommand::Connect { address, username, password } =>
            {
                set_auth_variables(address, username, password)
                    .await?;
                client = Some(build_client()
                    .await?);
                println!("You are connected to {}. Cool!", client.clone().unwrap().cluster_name);
            }
        ReplCommand::Disconnect { test } => {
            println!("{test} ");
        }

        ReplCommand::About { } => {
            match client.as_ref().is_some() {
                true => {
                    let output = client.clone().unwrap().about().await?;
                    dbg!(output);
                }
                _ => {
                    println!("Client error, are you sure that you are connected?");
                }
            }
        }

        ReplCommand::Clone { node, source_vmid, dest_vmid, description, full, name, pool } => {
            let mut clone_args = HashMap::new();
            clone_args.insert("newid", Value::from(dest_vmid));
            if description.is_some() {clone_args.insert("description", Value::from(description)); }
            if full.is_some() { clone_args.insert("full", Value::from(full)); }
            if name.is_some() { clone_args.insert("name", Value::from(name)); }
            if pool.is_some() { clone_args.insert("pool", Value::from(pool)); }

            let output = client.clone().unwrap().clone(node, source_vmid, clone_args).await?;
            println!("done?");
        }

        ReplCommand::Quit => {
            println!("pce");
            exit(0);
        }
        }
    }

}

