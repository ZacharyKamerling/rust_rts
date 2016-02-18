#![allow(dead_code)]

extern crate time;
extern crate byteorder;

mod data;
mod jps;
mod netcom;
mod basic_unit;
mod basic_weapon;
mod basic_missile;
mod kdt;
mod movement;
mod bytegrid;
mod useful_bits;
mod setup_game;
mod units;

use std::time::Duration;
use std::thread::sleep;
use self::time::{PreciseTime};
use std::io::Cursor;
use self::byteorder::{WriteBytesExt, BigEndian};

use data::game::{Game};
use data::kdt_point::{populate_with_kdtunits,populate_with_kdtmissiles};
use data::aliases::*;
use setup_game::setup_game;

fn main() {
    //bytegrid::test();
    //jps::bench();
    //jps::test();
    //kdt::bench();
    //movement::test_circle_line_intersection();
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
    let fps: usize = 50;
    let message_frequency: usize = fps / 10;
    let units = {
        let mut tmp = Vec::new();
        tmp.push(units::test_unit::prototype());
        tmp
    };

	let mut game = Game::new(fps, 2048, 8, 256,256, units, Vec::new(), Vec::new());
    setup_game(&mut game);

    println!("Game started.");
    let mut loop_count: usize = 0;

	loop {
		let start_time = PreciseTime::now();
        // INCORPORATE PLAYER MESSAGES
        let player_msgs = netcom::get_messages(&mut netc);

        game.incorporate_messages(player_msgs);

        game.unit_kdt = populate_with_kdtunits(&game.units);
        game.missile_kdt = populate_with_kdtmissiles(&game.missiles);

        let unit_iterator = game.units.iter();
        let team_iterator = game.teams.iter();
        let misl_iterator = game.missiles.iter();

        // Step units one logical frame
        for &id in &unit_iterator {
            if game.units.progress[id] >= game.units.progress_required[id] {
                basic_unit::event_handler(&mut game, UnitEvent::UnitSteps(id));
            }
        }

        for &id in &unit_iterator {
            if game.units.progress[id] >= game.units.progress_required[id] {
                basic_unit::move_and_collide_and_correct(&mut game, id);
            }
        }

        // Send data to each team
        for &team in &team_iterator {
            // Clear visible units
            for &id in &unit_iterator {
                game.teams.visible[team][id] = false;
            }

            // For each unit, find visible units & missiles and set their flag
            for &id in &unit_iterator {
                if game.units.team[id] == team {
                    let vis_enemies = basic_unit::enemies_in_vision(&game, id);

                    for kdtp in vis_enemies {
                        game.teams.visible[team][kdtp.id] = true;
                    }

                    let vis_missiles = basic_unit::missiles_in_vision(&game, id);

                    for kdtp in vis_missiles {
                        game.teams.visible_missiles[team][kdtp.id];
                    }
                }
            }

            if loop_count % message_frequency == 0 {
                let msg_number = (loop_count / message_frequency) as u32;
                let mut unit_msg = Cursor::new(Vec::new());
                // Message #
                let _ = unit_msg.write_u32::<BigEndian>(msg_number as u32);
                // CONVERT UNITS INTO DATA PACKETS
                for &id in &unit_iterator {

                    if game.units.team[id] == team || game.teams.visible[team][id] {
                        basic_unit::encode(&mut game, id, &mut unit_msg);
                    }
                }

                let mut misl_msg = Cursor::new(Vec::new());
                let _ = misl_msg.write_u32::<BigEndian>(msg_number as u32);


                for &id in &misl_iterator {
                    if game.teams.visible_missiles[team][id] {
                        basic_missile::encode(&mut game, id, &mut misl_msg);
                    }
                }

                let team_usize = unsafe {
                    team.usize_unwrap()
                };

                netcom::send_message_to_team(netc.clone(), unit_msg.into_inner(), team_usize);
                netcom::send_message_to_team(netc.clone(), misl_msg.into_inner(), team_usize);
            }
        }

        loop_count += 1;
		let end_time = PreciseTime::now();
		let time_spent = start_time.to(end_time).num_milliseconds();

        if (1000 / fps as i64) - time_spent > 0 {
            sleep(Duration::from_millis(((1000 / fps as i64) - time_spent) as u64));
        }
        else {
            println!("Logic is laggy: {}", loop_count);
        }
	}
}