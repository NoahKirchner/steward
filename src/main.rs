mod cli;
use cli::*;
mod auth;
use auth::*;
use std::process::exit;
use tokio;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut rl = Repl::<Cli>::new();
    loop {
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command.command {
            ReplCommand::Connect { address, username, password } =>
            {
                set_auth_variables(address, username, password)
                    .await?;
            }
        ReplCommand::Disconnect { test } => {
            println!("{test} ");
        }
        ReplCommand::Quit => {
            println!("pce");
            exit(0);
        }
        }
    }

}

