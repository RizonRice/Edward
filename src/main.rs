extern crate chrono;
extern crate irc;

use chrono::offset::local::Local;
use irc::client::prelude::*;
use std::collections::BTreeMap;
use std::default::Default;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command as SysCommand;

fn get_reply_target(prefix: Option<String>, target: &str, nickname: &str) -> String {
    if target == nickname {
        let prefix = prefix.unwrap_or("".to_owned());
        let target_nick = prefix.split('!').nth(0).unwrap_or("");
        String::from(target_nick)
    } else {
        String::from(target)
    }
}

fn get_commands(list_path: &Path) -> Result<BTreeMap<String, PathBuf>, io::Error> {
    if ! list_path.is_file() {
        panic!("Path does not point to a file");
    }

    let directory = list_path.parent().unwrap();
    try!(env::set_current_dir(directory));

    let filename = list_path.file_name().expect("Invalid filename");
    let file = try!(OpenOptions::new().read(true).open(filename));
    let file_reader = BufReader::new(&file);

    let mut commands: BTreeMap<String, PathBuf> = BTreeMap::new();

    for line in file_reader.lines().flat_map(Result::ok) {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }
        let command = line.split_whitespace().nth(0).unwrap();
        let path = line.split_whitespace().skip(1).collect::<PathBuf>();

        commands.insert(String::from(command), path);
    }

    Ok(commands)
}

fn run_command(command: &Path) -> Option<String> {
    match SysCommand::new("sh").arg("-c").arg(command).output() {
        Ok(output) => {
            String::from_utf8(output.stdout).ok()
        }
        Err(_) => None,
    }
}

fn main() {
    let commands = get_commands(Path::new("commands/list.txt")).expect("Failed to get commands list");

    let nickname = "rustmachine".to_string();
    let server = "irc.rizon.net".to_string();
    let channels = vec!["#lunarmage".to_string()];

    let trigger = '.';

    let cfg = Config {
        nickname: Some(nickname.to_owned()),
        server: Some(server),
        channels: Some(channels),
        .. Default::default()
    };

    let server = IrcServer::from_config(cfg).unwrap();
    server.identify().expect("Failed to identify with IRC server");

    for message in server.iter() {
        if let Ok(reply) = message {
            let time = Local::now().format("[%d/%m/%y %T]");
            let display_message = format!("{} {}", time, reply.to_string());

            print!("{}", &display_message);

            match reply.command {
                Command::PRIVMSG(target, message) => {
                    let message = message.trim_right();
                    if target == server.current_nickname() {
                        let target = get_reply_target(reply.prefix, &target, &server.current_nickname());
                        if message.starts_with(trigger) {
                            if let Some(func) = commands.get(&message.split_whitespace().nth(0).unwrap_or("")[1..]) {
                                if let Some(output) = run_command(&func) {
                                    let _ = server.send_privmsg(&target, &output);
                                }
                            }
                        } else {
                            if let Some(func) = commands.get(&message.split_whitespace().nth(0).unwrap_or("")[0..]) {
                                if let Some(output) = run_command(&func) {
                                    let _ = server.send_privmsg(&target, &output);
                                }
                            }
                        }
                    } else {
                        if message.starts_with(trigger) {
                            if let Some(func) = commands.get(&message.split_whitespace().nth(0).unwrap_or("")[1..]) {
                                if let Some(output) = run_command(&func) {
                                    let _ = server.send_privmsg(&target, &output);
                                }
                            }
                        }
                    }
                },
                _ => {},
            }
        }
    }
}
