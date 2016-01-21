#![allow(dead_code)]

extern crate time;
extern crate byteorder;

mod data;
mod jps;
mod netcom;
mod basic;
mod kdt;
mod movement;
mod bytegrid;
mod useful_bits;
mod setup_game;

use std::collections::HashSet;
use std::thread::sleep_ms;
use self::time::{PreciseTime};
use std::io::Cursor;
use self::byteorder::{WriteBytesExt, BigEndian};

use data::game::{Game};
use data::kdt_point::populate_with_kdtpoints;
use data::aliases::*;
use setup_game::setup_game;

fn main() {
    //bytegrid::test();
    //jps::bench();
    //kdt::bench();
    main_main();
}

fn main_main() {
    println!("Networking.");
	let address = "fe80::5937:6c62:d988:22a7%2".to_string();
	let port = "4444".to_string();
	let players =
		[ ("p1".to_string(), "p1".to_string(), 0)
	    , ("p2".to_string(), "p2".to_string(), 0)
	    , ("p3".to_string(), "p3".to_string(), 1)
	    , ("p4".to_string(), "p4".to_string(), 1)
	    ];
	let mut netc = netcom::new(&players, port, address);
    println!("Game.");
	let mut game = Game::new(2048, 256,256);
    setup_game(&mut game);
    println!("Game started.");

    let mut loop_count: u32 = 0;

	loop {
		let start_time = PreciseTime::now();
        // INCORPORATE PLAYER MESSAGES
        let player_msgs = netcom::get_messages(&mut netc);

        //game.incorporate_messages(player_msgs);

        game.kdt = populate_with_kdtpoints(&game.units);

        // STEP UNITS ONE LOGICAL FRAME
		for id in 0..game.units.alive.len() {
            if game.units.alive[id] {
                basic::event_handler(&mut game, UnitEvent::UnitSteps(id));
            }
		}

        // SEND DATA TO EACH TEAM
        for team in 0..game.teams.visible.len() {
            let mut msg = Cursor::new(Vec::new());
            let mut set = HashSet::with_capacity(512);

            let _ = msg.write_u32::<BigEndian>(loop_count);

            // UPDATE KDT FOR TEAM
            for id in 0..game.units.alive.len() {
                if game.units.alive[id] {
                    let vis_enemies = basic::enemies_in_range(&game, game.units.sight_range[id], id, true, true, true);
                    for kdtp in vis_enemies {
                        set.insert(kdtp.id);
                    }
                }
            }

            // CONVERT UNITS INTO DATA PACKETS
            for id in 0..game.units.alive.len() {
                if game.units.alive[id] && (set.contains(&id) || game.units.team[id] == team) {
                    basic::encode(&mut game, id, &mut msg);
                }
            }

            netcom::send_message_to_team(&mut netc, msg.into_inner(), team);

            game.teams.visible[team] = set;
        }

        //println!("{} : {} : {:?}", game.units.x[0], game.units.y[0], game.units.path[0].len());

        loop_count += 1;
		let end_time = PreciseTime::now();
		let time_spent = start_time.to(end_time).num_milliseconds();

        if 100 - time_spent > 0 {
            sleep_ms(100 - time_spent as u32);
        }
	}
}