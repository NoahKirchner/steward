use std::process::{exit};
use std::borrow::Cow;
use std::{marker::PhantomData};
use console::style;
use rustyline::{EventHandler, RepeatCount, ConditionalEventHandler, EventContext};
use rustyline::completion::Completer;

use clap::{Parser, Subcommand, arg, ValueEnum, CommandFactory};
use rustyline::{DefaultEditor, error::ReadlineError, highlight::Highlighter, hint::Hinter,
validate::Validator, Cmd, Editor, Event, Helper, KeyCode, KeyEvent, Modifiers};

use colored::{Colorize, ColoredString};



struct ReplHelper<C: Parser> {
    c_phantom: PhantomData<C>
}

impl<C:Parser> Completer for ReplHelper<C> {
    type Candidate = &'static str;
}

impl<C: Parser> Highlighter for ReplHelper<C> {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(style(hint).dim().to_string())
    }
}

impl<C: Parser> Hinter for ReplHelper<C> {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let command = C::command();
        let args = shlex::split(line).unwrap();
        if let [arg] = args.as_slice() {
            for c in command.get_subcommands() {
                if let Some(x) = c.get_name().strip_prefix(arg) {
                    return Some(x.to_string());
                }
            }
        }
        None
    }
}

impl<C: Parser> Validator for ReplHelper<C> {}
impl<C: Parser> Helper for ReplHelper<C> {}

struct TabEventHandler;
impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {

        if ctx.line()[..ctx.pos()]
            .chars()
                .rev()
                .next()
                .is_none()
        {
            println!();
            let mut cmd = Cli::command();
            let _ = cmd.print_long_help();
            Some(Cmd::AcceptLine)
        } else {
            None
        }
    }
}

pub struct Repl<C: Parser> {
    rl: Editor<ReplHelper<C>, rustyline::history::FileHistory>,
}

impl<C: Parser> Repl<C> {
    pub fn new() -> Self {
        let mut rl = Editor::<ReplHelper<C>, _>::new().unwrap();
        rl.set_helper(Some(ReplHelper {
            c_phantom: PhantomData,
        }));
        rl.bind_sequence(
            Event::KeySeq(vec![KeyEvent(KeyCode::Tab, Modifiers::NONE)]),
            Cmd::CompleteHint);
        rl.bind_sequence(
            Event::KeySeq(vec![KeyEvent(KeyCode::Tab, Modifiers::NONE)]),

            EventHandler::Conditional(Box::new(TabEventHandler)));
        Repl { rl }
    }

    pub fn read_command(&mut self) -> Option<C> {
        let line = match self.rl.readline(&style("STEWARD > ").green().bold().bright().to_string()) {
            Ok(x) => x,
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof |
                rustyline::error::ReadlineError::Interrupted => exit(0),
                rustyline::error::ReadlineError::WindowResized => return None,
                _ => panic!("Error in read line: {e:?}"),
            },
        };
        if line.trim().is_empty() {
            return None;
        }
        _ = self.rl.add_history_entry(line.as_str());
        let split_line = shlex::split(&line).unwrap();
        match C::try_parse_from(Some("".to_owned()).into_iter().chain(split_line)) {
            Ok(c) => Some(c),
            Err(e) => {
                e.print().unwrap();
                None
            }
        }
    }
}

#[derive(Parser)]
#[command(author = "Noah Kirchner", version = "0.1", about = "Proxmox Range Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: ReplCommand,
}


#[derive(Debug, Subcommand, PartialEq)]
#[command(name="")]
pub enum ReplCommand {
    #[command(about = "Connects to a Proxmox instance via credentials. This will negotiate an API key that is saved in an environment variable, so you should only need to run this once per terminal (bash, fsh, etc.) session")]
    Connect {
       
        #[arg(help = "IP Address or URL of the Proxmox server to authenticate to. ex. (https://xxxxxx:xxxx)")]
        address:String,

        #[arg(help = "The username to authenticate as in the format user@realm. ex. (joe@pam)")]
        username:String,
      
        #[arg(help = "The password to authenticate with")]
        password: String,

        /* 
         * @TODO Optional command (with default) for API key expiration 
         */

    },
    #[command(about = "Deletes the API key created by connect and removes it from your environment variables.")]
    Disconnect {
        test: String
    },
    #[command(alias="exit")]
    Quit,
}

