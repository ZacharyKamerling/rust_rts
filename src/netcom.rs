extern crate websocket;

use std::sync::{Arc, Mutex};
use std::thread;
use self::websocket::{Server, Message};

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