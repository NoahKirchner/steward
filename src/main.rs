mod cli;
use cli::*;
mod auth;
use auth::*;
mod client;
use client::*;
use std::process::exit;
use tokio;
use std::error::Error;

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
                    client.clone().unwrap().about().await?;
                }
                _ => {
                    println!("Client error, are you sure that you are connected?");
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

