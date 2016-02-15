#![allow(dead_code)]

extern crate time;
extern crate byteorder;

mod data;
mod jps;
mod netcom;
mod basic_unit;
mod basic_weapon;
mod kdt;
mod movement;
mod bytegrid;
mod useful_bits;
mod setup_game;

use std::time::Duration;
use std::thread::sleep;
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
    //jps::test();
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
    let fps: usize = 20;
    let message_frequency: usize = fps / 10;
	let mut game = Game::new(fps, 2048, 8, 256,256);
    setup_game(&mut game);

    println!("Game started.");
    let mut loop_count: usize = 0;

	loop {
		let start_time = PreciseTime::now();
        // INCORPORATE PLAYER MESSAGES
        let player_msgs = netcom::get_messages(&mut netc);

        game.incorporate_messages(player_msgs);

        game.kdt = populate_with_kdtpoints(&game.units);

        let unit_iterator = game.units.iter();
        let team_iterator = game.teams.iter();

        // Step units one logical frame
        for &id in &unit_iterator {
            if game.units.progress[id] >= game.units.progress_required[id] {
                basic_unit::event_handler(&mut game, UnitEvent::UnitSteps(id));
            }
        }

        // Send data to each team
        for &team in &team_iterator {
            // Clear visible units
            for &id in &unit_iterator {
                game.teams.visible[team][id] = false;
            }

            // For each unit, find visible units and set their flag
            for &id in &unit_iterator {
                if game.units.team[id] == team {
                    let vis_enemies = basic_unit::enemies_in_vision(&game, id);

                    for kdtp in vis_enemies {
                        game.teams.visible[team][kdtp.id] = true;
                    }
                }
            }

            if loop_count % message_frequency == 0 {
                let mut msg = Cursor::new(Vec::new());
                // Message #
                let _ = msg.write_u32::<BigEndian>((loop_count / message_frequency) as u32);
                // CONVERT UNITS INTO DATA PACKETS
                for &id in &unit_iterator {

                    if game.teams.visible[team][id] || game.units.team[id] == team {
                        basic_unit::encode(&mut game, id, &mut msg);
                    }
                }

                let team_usize = unsafe {
                    team.usize_unwrap()
                };

                netcom::send_message_to_team(&mut netc, msg.into_inner(), team_usize);
            }
        }

        loop_count += 1;
		let end_time = PreciseTime::now();
		let time_spent = start_time.to(end_time).num_milliseconds();

        if (1000 / fps as i64) - time_spent > 0 {
            sleep(Duration::from_millis(((1000 / fps as i64) - time_spent) as u64));
        }
        else {
            println!("Logic is laggy.");
        }
	}
}