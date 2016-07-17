extern crate chrono;
extern crate irc;

use chrono::offset::local::Local;
use irc::client::prelude::*;
use std::default::Default;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// fn is_command(trigger: char, message: &str) -> bool {
//     match message.chars().nth(0) {
//         Some(character) => {
//             match character == trigger {
//                 true => true,
//                 false => false,
//             }
//         }
//         None => false,
//     }
// }

fn get_reply_target(prefix: Option<String>, target: &str, nickname: &str) -> String {
    if target == nickname {
        let prefix = prefix.unwrap_or("".to_owned());
        let target_nick = prefix.split('!').nth(0).unwrap_or("");
        String::from(target_nick)
    } else {
        String::from(target)
    }
}

fn bots(server: &IrcServer, target: &str) -> Result<(), io::Error> {
    try!(server.send_privmsg(target, "Reporting in! [Rust]"));
    Ok(())
}

fn write_to_file(path: &Path, rx: Receiver<String>) -> Result<(), io::Error> {
    let mut logfile = try!(OpenOptions::new()
        .create(true)
        .append(true)
        .open(path));

    for line in rx {
        let _ = logfile.write(line.as_bytes());
    }

    Ok(())
}

fn main() {
    let nickname = "rustmachine".to_string();
    let server = "irc.rizon.net".to_string();
    let channels = vec!["#lunarmage".to_string()];

    let trigger = '.';
    let bots_command = format!("{}{}", trigger, "bots");

    let cfg = Config {
        nickname: Some(nickname.to_owned()),
        server: Some(server),
        channels: Some(channels),
        .. Default::default()
    };

    let server = IrcServer::from_config(cfg).unwrap();
    server.identify().expect("Failed to identify with IRC server");

    let (tx, rx) = channel::<String>();
    thread::spawn(|| {
        let _ = write_to_file(Path::new("logfile"), rx);
    });

    for message in server.iter() {
        if let Ok(reply) = message {
            let tx = tx.clone();
            let time = Local::now().format("[%d/%m/%y %T]");
            let display_message = format!("{} {}", time, reply.to_string());

            print!("{}", &display_message);
            let _ = tx.send(display_message);

            match reply.command {
                Command::PRIVMSG(target, message) => {
                    let message = message.trim_right();
                    if target == server.current_nickname() {
                        if message == "bots" || message == bots_command.as_str() {
                            let target = get_reply_target(reply.prefix.to_owned(), &target, server.current_nickname());
                            let _ = bots(&server, &target);
                        }
                    } else {
                        if message == bots_command.as_str() {
                            let target = get_reply_target(reply.prefix.to_owned(), &target, server.current_nickname());
                            let _ = bots(&server, &target);
                        }
                    }
                },
                _ => {},
            }
        }
    }
}
