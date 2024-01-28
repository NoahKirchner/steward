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
        let mut prompt:String = "STEWARD > ".to_string();
        if client.clone().is_some() { 
            prompt = format!("STEWARD @ {} > ", client.clone().unwrap().cluster_name.replace("\"", ""));
        }
        let Some(command) = rl.read_command(prompt) else {
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

        ReplCommand::Clone { bulk, bulk_vmid, batch, batch_root, batches, node, source_vmid, dest_vmid, description, full, name, pool } => {
            let mut clone_args = HashMap::new();
 
            if full.is_some() { clone_args.insert("full", Value::from(full)); }
            if description.is_some() {clone_args.insert("description", Value::from(description)); }
            if pool.is_some() { clone_args.insert("pool", Value::from(pool)); } 

            if bulk == true && bulk_vmid.is_some() {
                for vmid in bulk_vmid.unwrap() .. dest_vmid+1 {
                    // Clones the HashMap so we can add values into it that are only in 
                    // scope for this loop.
                    let mut _clone_args = clone_args.clone();
                    _clone_args.insert("newid", Value::from(vmid));
                    if name.is_some() { _clone_args.insert("name", Value::from(format!("{}-{}", name.clone().unwrap(), vmid)));} 
                let _output = client.clone().unwrap().clone_vm(node.clone(), source_vmid.clone(), _clone_args).await?;
                }
            } 
            if batch == true && batch_root.is_some() && batches.is_some() {
                let padding_size = batches.clone().unwrap().to_string().len();
                for batch in 0 .. batches.unwrap()+1 {
                    // Constructs a correctly padded batch VMID as a string and then parses it to
                    // i32
                    let vmid = format!("{}{:0padding_size$}{}", 
                                           batch_root.unwrap(), 
                                           batch,
                                           dest_vmid,
                                           padding_size = padding_size
                                           ).parse::<i32>().unwrap();
                    let mut _clone_args = clone_args.clone();
                    _clone_args.insert("newid", Value::from(vmid));
                    
                    if name.is_some() { _clone_args.insert("name", Value::from(format!("{}-{}", name.clone().unwrap(), batch))); }
                    
                    
                    let _output = client.clone().unwrap().clone_vm(node.clone(), source_vmid.clone(), _clone_args).await?;

                }
            }
            else 
            {

              
    

            clone_args.insert("newid", Value::from(dest_vmid));
            if name.is_some() { clone_args.insert("name", Value::from(name)); }

            // Match to make sure client is real TODO
            let _output = client.clone().unwrap().clone_vm(node, source_vmid, clone_args).await?;
            }
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

