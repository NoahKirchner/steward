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

    let client_error = "You do not appear to have connected to a proxmox cluster. Please run 'connect' before you use this command. Thanks!";

    let mut rl = Repl::<Cli>::new();
    let mut client: Option<StewardClient> = Option::None;
    loop {


        let mut prompt:String = "STEWARD > ".to_string();
        
        // Checks the existence of the client for each loop iteration
        match &client {
            Some(x) => {
                prompt = format!("STEWARD @ {} > ", &x.cluster_name);
            }
            // TODO add get_auth_variables based connection here
            None => (),
        }


        let Some(command) = rl.read_command(prompt) else {
            continue;
        };


        match command.command {

            ReplCommand::Connect { address, username, password } =>
            {

                // Defined in auth.rs
                set_auth_variables(address, username, password).await?;

                match build_client().await {
                    Ok(x) => {
                        println!("You are connected to {}.", &x.cluster_name);
                        client = Some(x);
                    }
                    // TODO error handling
                    Err(_) => todo!()
                }
            }


        ReplCommand::Disconnect { test } => {
            println!("{test} ");
        }


        ReplCommand::About { } => {

            match &client {
                Some(x) => {
                    let output = &x.about().await?;
                    //TODO pretty printing
                    dbg!(output);
                }
                None => {
                    println!("{}", client_error)
                }
            }


        }

        // This command only seems to have issues in bulk mode at massive numbers with a few 
        // 500 internal server errors sneaking through. Handling this is going to be part of the
        // job/task system, but just noting it down here for future reference. TODO 
        ReplCommand::Clone { bulk, bulk_vmid, batch, batch_root, batches, node, source_vmid, dest_vmid, description, full, name, pool } => {
            let mut clone_args = HashMap::new();
 
            if full.is_some() { clone_args.insert("full", Value::from(full)); }
            if description.is_some() {clone_args.insert("description", Value::from(description)); }
            if pool.is_some() { clone_args.insert("pool", Value::from(pool)); } 

            // Probably a less cancerous way to do this 
            if bulk == true && bulk_vmid.is_some() {
                for vmid in bulk_vmid.unwrap() .. dest_vmid+1 {
                    // Clones the HashMap so we can add values into it that are only in 
                    // scope for this loop.
                    let mut _clone_args = clone_args.clone();
                    _clone_args.insert("newid", Value::from(vmid));

                    // just a name check.
                    match &name {
                        Some(x) => {
                            _clone_args.insert("name", Value::from(format!("{}-{}", x, vmid)));
                        }
                        None => {
                            ();
                        }
                    }


                    let _output = match &client {
                        Some(x) => {
                            // TODO make this not retarded and use _output somewhere useful
                            &x.clone_vm(node.to_owned(), source_vmid.to_owned(), _clone_args).await?;
                        }
                        None => {
                            println!("{}", client_error);
                        }
                    };
                    dbg!(_output);
                }

            } 

            else if batch == true && batch_root.is_some() && batches.is_some() {

                // Returns padding value and total number of batches together in a tuple, 
                // also handles error checking i guess
                let (padding_size, batches) = match batches {
                    Some(x) => {
                        (x.clone().to_string().len(), x)
                    }
                    None => {
                        // Actual error handling;
                        panic!("Aw shit that aint workin");
                    }
                };

                for batch in 0 .. batches+1 {
                    // Constructs a correctly padded batch VMID as a string and then parses it to
                    // i32. This is horrific.
                    let vmid = format!("{}{:0padding_size$}{}", 
                                           batch_root.unwrap(), 
                                           batch,
                                           dest_vmid,
                                           padding_size = padding_size
                                           ).parse::<i32>().unwrap();

                    // may God forgive me for a .clone(), but it makes sense in this context
                    let mut _clone_args = clone_args.clone();

                    _clone_args.insert("newid", Value::from(vmid));
                    
                    match &name {
                       Some(x) => {
                           _clone_args.insert("name", Value::from(format!("{}-{}", x, batch)));
                       }
                       None => {
                           ();
                       }
                    }

                    
                    let _output = match &client {
                        Some(x) => {
                            // TODO make this not retarded and use _output somewhere useful
                            &x.clone_vm(node.to_owned(), source_vmid.to_owned(), _clone_args).await?;
                        }
                        None => {
                            println!("{}", client_error);
                        }
                    };
                    dbg!(_output);
                }
            }
            else 
            {


            clone_args.insert("newid", Value::from(dest_vmid));
            if name.is_some() { clone_args.insert("name", Value::from(name)); }

            // Match to make sure client is real TODO
            let _output = match &client {
                Some(x) => {
                    &x.clone_vm(node, source_vmid, clone_args).await?;
                }
                None => {
                    println!("{}", client_error);
                }
            };
            }
            }

        ReplCommand::Destroy { bulk, node, dest_vmid, destroy_disks, purge_jobs } => {
            let mut destroy_args = HashMap::new();
            if destroy_disks.is_some() {destroy_args.insert("destroy-unreferenced-disks", Value::from(destroy_disks)); }
            if purge_jobs.is_some() {destroy_args.insert("purge", Value::from(purge_jobs)); }

            if bulk.is_some() {
                for vmid in bulk.unwrap() .. dest_vmid+1 {
                    let _output = match &client { 
                        Some(x) => {
                            &x.destroy_vm(node.clone(), vmid, destroy_args.clone()).await?;
                        }
                        None => {
                            println!("{}", client_error);
                        }
                    };

                }
            }

            let _output = match &client {
                Some(x) => {
                    &x.destroy_vm(node, dest_vmid, destroy_args).await?;
                }
                None => {
                    println!("{}", client_error);
                }
            };
        
        }

        ReplCommand::Status { node, vmid } => {
            let _output = match &client {
                Some(x) => {
                    &x.vm_status(node, vmid).await?;
                }
                None => {
                    println!("{}", client_error);
                }
            };
        }

        ReplCommand::Quit => {
            println!("pce");
            exit(0);
        }
        }
    }

}

