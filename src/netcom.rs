extern crate websocket;
extern crate byteorder;

use std::string::String;
use std::io::Cursor;
use std::io::Read;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::sync::{Arc, Mutex};
use std::thread;
use self::websocket::{Server, Message};

type TeamID = usize;
type Name = String;

struct Netcom {
    players: Vec<Player>,
    messages: Vec<(String, TeamID, Vec<u8>)>,
}
struct Player;

fn test() {
    let players = Arc::new(Mutex::new(<Vec<Player>>::new()));
    let server = Server::bind("127.0.0.1:1234").unwrap();

    for connection in server {
        // Spawn a new thread for each connection.
        let players = players.clone();
        thread::spawn(move || {
            let request = connection.unwrap().read_request().unwrap(); // Get the request
            let response = request.accept(); // Form a response
            let mut client = response.send().unwrap(); // Send the response

            let message = Message::Text("Hello, client!".to_string());
            let _ = client.send_message(message);

            let con_info = client.recv_message().unwrap();

            match con_info {
                Message::Binary(_) => {

                }
                _ => return,
            }

            {
                let mut players = players.lock().unwrap();
                players.push(Player);
            }

            for message in client.incoming_messages::<Message>() {
                println!("Recv: {:?}", message.unwrap());
            }
        });
    }
}

fn name_and_pass(data: &mut Cursor<&[u8]>) -> (String,String) {
    let name = text(data);
    let pass = text(data);
    (name,pass)
}

fn text(data: &mut Cursor<&[u8]>) -> String {
    let text_len = data.read_u16::<BigEndian>().unwrap();
    let mut buff = Vec::new();
    let _ = data.take(text_len as u64).read_to_end(&mut buff);
    String::from_utf8(buff).unwrap()
}