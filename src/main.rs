mod cli;
use cli::*;
mod auth;
use auth::*;
mod client;
use client::*;
use serde_json::Value;
use std::error::Error;
use std::{collections::HashMap, process::exit};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /*
     * Until I find a better solution we are just going to do a timeout
     */

    let client_error = "You do not appear to have connected to a proxmox cluster. Please run 'connect' before you use this command. Thanks!";

    let mut rl = Repl::<Cli>::new();
    let mut client: Option<StewardClient> = Option::None;
    loop {
        let mut prompt: String = "STEWARD > ".to_string();

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
            ReplCommand::Connect {
                address,
                username,
                password,
            } => {
                // Defined in auth.rs
                set_auth_variables(address, username, password).await?;

                match build_client().await {
                    Ok(x) => {
                        println!("You are connected to {}.", &x.cluster_name);
                        client = Some(x);
                    }
                    // TODO error handling
                    Err(_) => todo!(),
                }
            }

            ReplCommand::Disconnect { test } => {
                println!("{test} ");
            }

            ReplCommand::About {} => {
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
            ReplCommand::Clone {
                action,
                start_vmid,
                batches,
                node,
                source_vmid,
                dest_vmid,
                description,
                full,
                name,
                pool,
            } => {
                let mut clone_args = HashMap::new();

                if full.is_some() {
                    clone_args.insert("full", Value::from(full));
                }
                if description.is_some() {
                    clone_args.insert("description", Value::from(description));
                }
                if pool.is_some() {
                    clone_args.insert("pool", Value::from(pool));
                }

                // Verifies that a client exists
                match &client {
                    Some(_client) => {
                        match action {
                            // Checks if it was a bulk clone action.
                            Some(CloneAction::Bulk) => {
                                for vmid in start_vmid.unwrap()..dest_vmid + 1 {
                                    // Creates a copy of the function arguments
                                    let mut _clone_args = clone_args.clone();
                                    _clone_args.insert("newid", Value::from(vmid));
                                    // Just formats a name if it exists (TODO maybe just make this an if
                                    // statement)
                                    match &name {
                                        Some(_name) => {
                                            _clone_args.insert(
                                                "name",
                                                Value::from(format!("{}-{}", &_name, vmid)),
                                            );
                                        }
                                        None => {
                                            ();
                                        }
                                    }

                                    let _output = &_client
                                        .clone_vm(
                                            node.to_owned(),
                                            source_vmid.to_owned(),
                                            _clone_args,
                                        )
                                        .await?;
                                }
                            }
                            // Checks if it was a batch clone action
                            Some(CloneAction::Batch) => {
                                // Returns a tuple of the padding size for the batch number (so you have
                                // 000-250 instead of 0-250 for VMID purposes)
                                let (padding_size, batches) = match batches {
                                    Some(_batches) => {
                                        (_batches.clone().to_string().len(), _batches)
                                    }
                                    None => {
                                        panic!("Aw shit");
                                    }
                                };

                                for batch in 0..batches + 1 {
                                    // Formatting from the earlier comment to create a VMID with correct
                                    // padding
                                    let vmid = format!(
                                        "{}{:0padding_size$}{}",
                                        start_vmid.unwrap(),
                                        batch,
                                        dest_vmid,
                                        padding_size = padding_size
                                    )
                                    .parse::<i32>()
                                    .unwrap();

                                    let mut _clone_args = clone_args.clone();
                                    _clone_args.insert("newid", Value::from(vmid));

                                    match &name {
                                        Some(_name) => {
                                            _clone_args.insert(
                                                "name",
                                                Value::from(format!("{}-{}", &_name, batch)),
                                            );
                                        }
                                        None => {
                                            ();
                                        }
                                    }

                                    let _output = &_client
                                        .clone_vm(
                                            node.to_owned(),
                                            source_vmid.to_owned(),
                                            _clone_args,
                                        )
                                        .await?;
                                }
                            }
                            // Triggers if this is just a normal single clone action
                            None => {
                                clone_args.insert("newid", Value::from(dest_vmid));
                                if name.is_some() {
                                    clone_args.insert("name", Value::from(name));
                                }
                                let _output =
                                    &_client.clone_vm(node, source_vmid, clone_args).await?;
                            }
                        }
                    }
                    None => {
                        println!("{}", client_error)
                    }
                }
            }

            ReplCommand::Destroy {
                bulk,
                node,
                dest_vmid,
                destroy_disks,
                purge_jobs,
            } => {
                let mut destroy_args = HashMap::new();
                if destroy_disks.is_some() {
                    destroy_args.insert("destroy-unreferenced-disks", Value::from(destroy_disks));
                }
                if purge_jobs.is_some() {
                    destroy_args.insert("purge", Value::from(purge_jobs));
                }

                if bulk.is_some() {
                    for vmid in bulk.unwrap()..dest_vmid + 1 {
                        let _output = match &client {
                            Some(x) => {
                                &x.destroy_vm(node.clone(), vmid, destroy_args.clone())
                                    .await?;
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

            ReplCommand::Config {
                node,
                vmid,
                net_device,
                bridge,
                mac,
                vlan,
            } => {
                dbg!("test in main");
                let mut net_config_args = HashMap::new();

                net_config_args.insert("bridge", Value::from(bridge));

                if mac.is_some() {
                    net_config_args.insert("macaddr", Value::from(mac));
                }
                if vlan.is_some() {
                    net_config_args.insert("tag", Value::from(vlan));
                }

                dbg!(&net_config_args);

                match &client {
                    Some(_client) => {
                        dbg!("client matched fr fr");
                        let _output = &_client
                            .set_vm_net_config(node, vmid, net_device.as_str(), net_config_args)
                            .await?;

                        dbg!(_output);
                    }
                    None => {
                        println!("{}", client_error)
                    }
                }
            }

            ReplCommand::Quit => {
                println!("pce");
                exit(0);
            }
        }
    }
}
