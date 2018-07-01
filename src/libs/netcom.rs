extern crate ws;

use std::collections::HashMap;
use self::ws::{Handler, Sender, Message, Error, ErrorKind, CloseCode, listen};
use std::sync::{Arc, Mutex};
use std::thread;

impl Handler for Player {
    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(txt) => {
                if !self.has_name {
                    self.name = txt.clone();
                    self.has_name = true;
                }
                else if !self.has_pass {
                    self.pass = txt.clone();
                    self.has_pass = true;
                }

                if self.has_name && self.has_pass && !self.verified {
                    if let Some((pass, team)) = self.names_passes_teams.get(&self.name) {
                        if self.pass == pass.to_owned() {
                            self.team = *team;
                            self.verified = true;
                            let mut netcom = self.netcom.lock().unwrap();
                            netcom.players.insert(self.name.to_owned(), self.to_owned());
                            println!("Accepting player: {:?}", self.name);
                            Ok(())
                        }
                        else {
                            let _ = self.out.close(CloseCode::Normal);
                            Err(Error::new(ErrorKind::Protocol, "Passwords didn't match."))
                        }
                    }
                    else {
                        Err(Error::new(ErrorKind::Protocol, "No matching name or password."))
                    }
                }
                else if self.verified {
                    println!("{:?}", txt.clone());
                    Ok(())
                }
                else {
                    Ok(())
                }
            }
            Message::Binary(vec) => {
                if self.verified {
                    let mut netcom = self.netcom.lock().unwrap();
                    netcom.messages.push((self.name.clone(), self.team, vec));
                    Ok(())
                }
                else {
                    Err(Error::new(ErrorKind::Protocol, "Player not verified."))
                }
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        let mut netcom = self.netcom.lock().unwrap();
        netcom.players.remove(&self.name);
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

#[derive(Clone)]
struct Player {
    netcom: Arc<Mutex<Netcom>>,
    names_passes_teams: Arc<HashMap<String, (String, usize)>>,
    verified: bool,
    has_name: bool,
    has_pass: bool,
    name: String,
    pass: String,
    team: usize,
    out: Sender,
}

pub struct Netcom {
    players: HashMap<String, Player>,
    messages: Vec<(String, usize, Vec<u8>)>,
}

pub fn get_messages(net: &Arc<Mutex<Netcom>>) -> Vec<(String, usize, Vec<u8>)> {
    let mut net = net.lock().unwrap();
    let vec = net.messages.clone();
    net.messages.clear();
    vec
}

pub fn send_message_to_player(net: Arc<Mutex<Netcom>>, msg: Vec<u8>, name: String) {
    let players = {
        let net = net.lock().unwrap();
        net.players.clone()
    };

    for (player_name, player) in players {
        if player_name == name {
            let bin_msg = Message::Binary(msg.clone());
            let _ = player.out.send(bin_msg);
        }
    }
}

pub fn send_message_to_team(net: Arc<Mutex<Netcom>>, msg: Vec<u8>, team: usize) {
    let players = {
        let net = net.lock().unwrap();
        net.players.clone()
    };

    for (_, player) in players {
        if player.team == team {
            let bin_msg = Message::Binary(msg.clone());
            let _ = player.out.send(bin_msg);
        }
    }
}

pub fn new(names_passes_teams: &[(String, String, usize)], port: &str, address: &str) -> Arc<Mutex<Netcom>> {
    let netcom = Arc::new(Mutex::new(Netcom {
        players: HashMap::new(),
        messages: Vec::new(),
    }));

    let mut npt_map = HashMap::new();

    for (name, pass, team) in names_passes_teams {
        npt_map.insert(name.to_owned(), (pass.to_owned(), team.to_owned()));
    }

    let return_netcom = netcom.clone();

    let mut listen_on = String::new();
    listen_on.push_str(address);
    listen_on.push_str(":");
    listen_on.push_str(port);

    thread::spawn(move || {
        let npt_map = Arc::new(npt_map);
        listen(listen_on, |out| {
            Player {
                netcom: netcom.clone(),
                names_passes_teams: npt_map.clone(),
                verified: false,
                has_name: false,
                has_pass: false,
                name: String::new(),
                pass: String::new(),
                team: 0,
                out: out,
            }
        }).unwrap()
    });

    return return_netcom;
}