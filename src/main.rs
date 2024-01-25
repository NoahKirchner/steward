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

            // Match to make sure client is real TODO
            let _output = client.clone().unwrap().clone_vm(node, source_vmid, clone_args).await?;
            println!("done?");
        }

        ReplCommand::Destroy { node, vmid, destroy_disks, purge_jobs } => {
            let mut destroy_args = HashMap::new();
            if destroy_disks.is_some() {destroy_args.insert("destroy-unreferenced-disks", Value::from(destroy_disks)); }
            if purge_jobs.is_some() {destroy_args.insert("purge", Value::from(purge_jobs)); }

            // Match to make sure client is real TODO
            let _output = client.clone().unwrap().destroy_vm(node, vmid, destroy_args).await?;
        
        }

        ReplCommand::Status { node, vmid } => {
            let _output = client.clone().unwrap().vm_status(node, vmid).await?;
        }

        ReplCommand::Quit => {
            println!("pce");
            exit(0);
        }
        }
    }

}

