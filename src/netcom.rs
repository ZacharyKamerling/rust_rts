extern crate websocket;
extern crate byteorder;

use std::string::String;
use std::io::Cursor;
use std::io::Read;
#[allow(unused_imports)]
use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::sync::{Arc, Mutex};
use std::thread;
use self::websocket::{Server, Sender, Receiver, Message, WebSocketStream};
use self::websocket::server::sender as snder;

pub struct Netcom {
    players: Vec<Player>,
    messages: Vec<(String, usize, Vec<u8>)>,
}

struct Player {
    name: String,
    pass: String,
    team: usize,
    client: Arc<Mutex<snder::Sender<WebSocketStream>>>,
}


pub fn get_messages(net: &mut Arc<Mutex<Netcom>>) -> Vec<(String, usize, Vec<u8>)> {
    let mut net = net.lock().unwrap();
    let vec = net.messages.clone();
    net.messages.clear();
    vec
}

pub fn new(names_passes_teams: &[(String,String,usize)], port: String, address: String) -> Arc<Mutex<Netcom>> {
    let netcom = Arc::new(Mutex::new(Netcom{players: Vec::new(), messages: Vec::new()}));
    let server = Server::bind(&*(address + ":" + &port)).unwrap();
    let names_passes_teams = Arc::new(names_passes_teams.to_vec());
    let return_netcom = netcom.clone();

    thread::spawn(move || {
        for connection in server {
            // Spawn a new thread for each connection.
            let names_passes_teams = names_passes_teams.clone();
            let netcom = netcom.clone();
            thread::spawn(move || {
                let request = connection.unwrap().read_request().unwrap();
                let (sender, mut receiver) = request.accept().send().unwrap().split();
                let sender = Arc::new(Mutex::new(sender));

                let con_info = receiver.recv_message().unwrap();

                println!("Somebody is trying to connect...");

                let mut validated = false;
                let mut valid_name = "".to_string();
                let mut valid_team = 0;

                match con_info {
                    Message::Binary(info) => {
                        let maybe_name_pass = name_and_pass(&mut Cursor::new(info));
                        match maybe_name_pass {
                            None => return,
                            Some((name,pass)) => {
                                for player in (*names_passes_teams).iter() {
                                    let (player_name, player_pass, player_team) = (*player).clone();
                                    if player_name == name && player_pass == pass {
                                        valid_name = player_name;
                                        valid_team = player_team;
                                        validated = true;
                                        let mut netcom = netcom.lock().unwrap();
                                        netcom.players.push(Player{name: name.clone(), pass: pass.clone(), team: player_team, client: sender.clone()});
                                        println!("{} connected with password \"{}\".", valid_name.clone(), pass);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let message = Message::Text("You fail!".to_string());
                        let mut sender = sender.lock().unwrap();
                        let _ = sender.send_message(message);
                        println!("Somebody failed to connect...");
                    }
                }

                if validated {
                    for message in receiver.incoming_messages::<Message>() {
                        match message.unwrap() {
                            Message::Binary(info) => {
                                let mut netcom = netcom.lock().unwrap();
                                netcom.messages.push((valid_name.clone(), valid_team, info));
                            }
                            _ => break,
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
    let _ = vec.extend(name_vec);

    let _ = vec.write_u16::<BigEndian>(pass_str.len() as u16);
    let _ = vec.extend(pass_vec);

    match name_and_pass(&mut Cursor::new(vec)) {
        Some((name,pass)) => assert!(name == name_str && pass == pass_str),
        None => println!("Failure!"),
    }
}