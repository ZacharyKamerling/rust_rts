#![allow(dead_code)]

extern crate time;

mod data;
mod jps;
mod netcom;
mod basic;
mod kdt;
mod movement;
mod bytegrid;
mod useful_bits;

use std::thread::sleep_ms;
use self::time::{PreciseTime};

use data::game::{Game};

fn main() {
	let address = "2601:603:4000:2030:5937:6c62:d988:22a7".to_string();
	let port = "4444".to_string();
	let players =
		[ ("Player1".to_string(), "Password1".to_string(), 1)
	    , ("Player2".to_string(), "Password2".to_string(), 1)
	    , ("Player3".to_string(), "Passowrd3".to_string(), 2)
	    , ("Player4".to_string(), "Passowrd4".to_string(), 2)
	    ];
	let netc = netcom::new(&players, port, address);

	let game = Game::new(1024,1024);

	loop {
		let start_time = PreciseTime::now();
		/* Do Things */
		for id in 0..game.units.alive.len() {
		}

		let end_time = PreciseTime::now();
		let time_spent = start_time.to(end_time).num_milliseconds();
		sleep_ms(100 - time_spent as u32);
	}
}