extern crate websocket;
extern crate byteorder;

use std::string::String;
use std::io::Cursor;
use std::io::Read;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::ops::{DerefMut};
use std::sync::{Arc, Mutex};
use std::thread;
use self::websocket::{Server, Message, Sender, Receiver, WebSocketStream};
use self::websocket::sender::Sender as sndr;
use self::websocket::message::Type;

pub struct Netcom {
    players: Vec<Player>,
    messages: Vec<(String, usize, Vec<u8>)>,
}

#[derive(Clone)]
struct Player {
    name: String,
    pass: String,
    team: usize,
    client: Arc<Mutex<sndr<WebSocketStream>>>,
}

pub fn send_message_to_team(net: Arc<Mutex<Netcom>>, msg: Vec<u8>, team: usize) {
    thread::spawn(move || {
        let players = {
            let net = net.lock().unwrap();
            net.players.clone()
        };
        let bin_msg = Message::binary(msg);

        for player in players {
            if player.team == team {
                let mut lock = player.client.lock().unwrap();
                let mut sender = lock.deref_mut();
                let _ = sender.send_message(&bin_msg);
            }
        }
    });
}

pub fn send_message_to_player(net: Arc<Mutex<Netcom>>, msg: Vec<u8>, name: String) {
    thread::spawn(move || {
        let players = {
            let net = net.lock().unwrap();
            net.players.clone()
        };
        let bin_msg = Message::binary(msg);

        for player in players {
            if player.name == name {
                let mut lock = player.client.lock().unwrap();
                let mut sender = lock.deref_mut();
                let _ = sender.send_message(&bin_msg);
            }
        }
    });
}

pub fn get_messages(net: &Arc<Mutex<Netcom>>) -> Vec<(String, usize, Vec<u8>)> {
    let mut net = net.lock().unwrap();
    let vec = net.messages.clone();
    net.messages.clear();
    vec
}

pub fn new(names_passes_teams: &[(String,String,usize)], port: &str, address: &str) -> Arc<Mutex<Netcom>> {
    println!("Connecting to {}:{}", address, port);
    let netcom = Arc::new(Mutex::new(Netcom{players: Vec::new(), messages: Vec::new()}));
    let server = Server::bind(&*(address.to_owned() + ":" + port)).unwrap();
    let names_passes_teams = Arc::new(names_passes_teams.to_vec());
    let return_netcom = netcom.clone();

    thread::spawn(move || {
        for connection in server {
            // Spawn a new thread for each connection.
            let names_passes_teams = names_passes_teams.clone();
            let netcom = netcom.clone();
            thread::spawn(move || {
                println!("Somebody is trying to connect...");
                let request = connection.unwrap().read_request().unwrap();
                let (sender, mut receiver) = request.accept().send().unwrap().split();
                let sender_arc = Arc::new(Mutex::new(sender));
                let con_info: Message = receiver.recv_message().unwrap();

                let mut validated = false;
                let mut valid_name = "".to_string();
                let mut valid_team = 0;

                match con_info.opcode {
                    Type::Binary => {
                        let maybe_name_pass = name_and_pass(&mut Cursor::new(con_info.payload.to_vec()));
                        match maybe_name_pass {
                            None => return,
                            Some((name,pass)) => {
                                for player in &*names_passes_teams {
                                    let (player_name, player_pass, player_team) = player.clone();
                                    if player_name == name && player_pass == pass {
                                        valid_name = player_name;
                                        valid_team = player_team;
                                        validated = true;
                                        let mut netcom = netcom.lock().unwrap();
                                        netcom.players.push(Player{name: name.clone(), pass: pass.clone(), team: player_team, client: sender_arc.clone()});
                                        println!("{} connected with password \"{}\".", valid_name.clone(), pass);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        println!("Somebody failed to connect...");
                    }
                }

                if validated {
                    for possible_message in receiver.incoming_messages() {
                        match possible_message {
                            Ok(msg) => {
                                let message: Message = msg;
                                if let Type::Binary = message.opcode {
                                    let mut netcom = netcom.lock().unwrap();
                                    netcom.messages.push((valid_name.clone(), valid_team, message.payload.to_vec()));
                                }
                            }
                            _ => {
                                println!("Somebody disconnected...");
                                break;
                            }
                        }
                    }
                }
            });
        }
    });
    return_netcom
}

fn name_and_pass(data: &mut Cursor<Vec<u8>>) -> Option<(String,String)> {
    match text(data) {
        None => None,
        Some(name) => match text(data) {
            None => None,
            Some(pass) => Some((name,pass))
        }
    }
}

fn text(data: &mut Cursor<Vec<u8>>) -> Option<String> {
    let text_len = data.read_u16::<BigEndian>().unwrap();
    let mut buff = Vec::new();
    let _ = data.take(text_len as u64).read_to_end(&mut buff);

    match String::from_utf8(buff) {
        Ok(txt) => Some(txt),
        _       => None
    }
}

#[test]
fn name_and_pass_test() {
    let mut vec = Vec::new();
    let name_str = "Zach".to_string();
    let name_vec = name_str.as_bytes().to_vec();
    let pass_str = "Hoodabaga".to_string();
    let pass_vec = pass_str.as_bytes().to_vec();

    let _ = vec.write_u16::<BigEndian>(name_str.len() as u16);
    vec.extend(name_vec);

    let _ = vec.write_u16::<BigEndian>(pass_str.len() as u16);
    vec.extend(pass_vec);

    match name_and_pass(&mut Cursor::new(vec)) {
        Some((name,pass)) => assert!(name == name_str && pass == pass_str),
        None => println!("Failure!"),
    }
}