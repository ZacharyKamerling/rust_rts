#![allow(dead_code)]
#![feature(plugin)]
#![plugin(clippy)]
#![allow(modulo_one)]

extern crate core;
extern crate time;
extern crate byteorder;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod data;
mod pathing;
mod libs;
mod behavior;
mod useful_bits;
mod setup_game;
mod units;

use self::time::{PreciseTime};
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io;
use std::time::Duration;
use std::thread::sleep;
use std::io::Cursor;
use libs::netcom;

use data::game::{Game};
use data::logger;
use data::kdt_point as kdtp;
use data::aliases::*;
use setup_game::setup_game;

use behavior::missile::core as missile;
use behavior::unit::core as unit;
use behavior::weapon::core as weapon;

fn main() {
    //bytegrid::test();
    //pathing::path_grid::bench();
    //pathing::path_grid::test();
    //libs::kdt::bench();
    libs::fast_float::bench();
    //movement::test_circle_line_intersection();
    //main_main();
}

fn main_main() {
    let mut address = String::new();
    let mut port = String::new();

    println!("Enter your IP address");
    let _ = io::stdin().read_line(&mut address).unwrap();

    println!("Enter port number");
    let _ = io::stdin().read_line(&mut port).unwrap();

    address = address.trim().to_string();
    port = port.trim().to_string();

    println!("Networking.");
	let players =
		[ ("p1".to_string(), "p1".to_string(), 0)
	    , ("p2".to_string(), "p2".to_string(), 0)
	    , ("p3".to_string(), "p3".to_string(), 1)
	    , ("p4".to_string(), "p4".to_string(), 1)
	    ];

	let netc = netcom::new(&players, &port, &address);

    let units = vec!(
        units::test_unit::prototype(),
        units::test_structure::prototype(),
        );

    let weapons = vec!(
        units::test_unit::wpn_proto(),
        units::test_structure::wpn_proto(),
    );

    let missiles = vec!(
        units::test_unit::missile_proto(),
        units::test_structure::missile_proto(),
    );

	let mut game = &mut Game::new(4096, 8, (256, 256), units, weapons, missiles, netc);
    setup_game(game);

    println!("Game started.");
    let mut loop_count: u32 = 0;

	loop {
		let start_time = PreciseTime::now();
        let player_msgs = netcom::get_messages(&game.netcom);

        data::game::incorporate_messages(game, player_msgs);

        // STEP MISSILES
        for &id in &game.missiles.iter() {
            missile::step_missile(game, id);
        }

        // STEP UNITS
        let unit_iterator = game.units.iter();

        for &id in &unit_iterator {
            if game.units.progress(id) >= game.units.progress_required(id) {
                unit::event_handler(game, UnitEvent::UnitSteps(id));
            }
        }

        // MOVE AND COLLIDE UNITS
        for &id in &unit_iterator {
            if game.units.progress(id) >= game.units.progress_required(id) {
                unit::move_and_collide_and_correct(game, id);
            }
        }

        // STEP WEAPONS
        for &id in &game.weapons.iter() {
            let u_id = game.weapons.unit_id[id];

            if game.units.progress(u_id) >= game.units.progress_required(u_id) {
                weapon::attack_orders(game, id, u_id);
            }
        }

        game.unit_kdt = kdtp::populate_with_kdtunits(&game.units);
        game.missile_kdt = kdtp::populate_with_kdtmissiles(&game.missiles);

        for &team in &game.teams.iter() {
            // CLEAR VISIBLE UNITS
            for &id in &unit_iterator {
                game.teams.visible[team][id] = false;
            }

            // CLEAR VISIBLE MISSILES
            for &id in &game.missiles.iter() {
                game.teams.visible_missiles[team][id] = false;
            }

            // FIND VISIBLE UNITS AND MISSILES
            for &id in &unit_iterator {
                if game.units.team(id) == team {
                    let vis_enemies = kdtp::enemies_in_vision(game, id);

                    for kdtp in vis_enemies {
                        game.teams.visible[team][kdtp.id] = true;
                    }

                    let vis_missiles = unit::missiles_in_vision(game, id);

                    for kdtp in vis_missiles {
                        game.teams.visible_missiles[team][kdtp.id] = true;
                    }
                }
            }
        }
        game.frame_number = loop_count;
        encode_and_send_data_to_teams(game);

        // LOOP TIMING STUFF
        loop_count += 1;
		let end_time = PreciseTime::now();
		let time_spent = start_time.to(end_time).num_milliseconds();

        if (1000 / FPS as i64) - time_spent > 0 {
            sleep(Duration::from_millis(((1000 / FPS as i64) - time_spent) as u64));
        }
        else {
            println!("Logic is laggy. Loop# {}. Time (ms): {:?}", loop_count, time_spent);
        }
	}
}

fn encode_and_send_data_to_teams(game: &mut Game) {
    let team_iter = game.teams.iter();
    let frame_number = game.frame_number;

    for &team in &team_iter {
        let mut logg_msg = Cursor::new(Vec::new());
        let _ = logg_msg.write_u32::<BigEndian>(frame_number);
        logger::encode_missile_booms(game, team, &mut logg_msg);
        logger::encode_unit_deaths(game, team, &mut logg_msg);

        let team_usize = unsafe {
            team.usize_unwrap()
        };
        netcom::send_message_to_team(game.netcom.clone(), logg_msg.into_inner(), team_usize);
    }

    let unit_deaths_iter = game.logger.unit_deaths.to_vec();

    for &team in &team_iter {
        for &boom in &game.logger.missile_booms {
            if game.teams.visible_missiles[team][boom.id] {
                // NOTE! Sets exploded missiles visibility to false so they aren't encoded twice
                game.teams.visible_missiles[team][boom.id] = false;
            }
        }
        for &death in &unit_deaths_iter {
            if game.teams.visible[team][death.id] {
                // NOTE! Sets dead units visibility to false so they aren't encoded twice
                game.teams.visible[team][death.id] = false;
            }
        }
    }

    for &boom in &game.logger.missile_booms {
        game.missiles.kill_missile(boom.id);
    }

    for &death in &unit_deaths_iter {
        if game.units.is_structure(death.id) {
            match game.units.width_and_height(death.id) {
                Some((w,h)) => {
                    let team = game.units.team(death.id);
                    let (x,y) = game.units.xy(death.id);
                    let hw = w as f32 / 2.0;
                    let hh = h as f32 / 2.0;
                    let bx = (x - hw + 0.0001) as isize;
                    let by = (y - hh + 0.0001) as isize;

                    for xo in bx..bx + w {
                        for yo in by..by + h {
                            let point_val = game.bytegrid.get_point((xo,yo));
                            game.bytegrid.set_point(point_val - 1, (xo,yo));
                            game.teams.jps_grid[team].open_point((xo,yo));
                        }
                    }
                }
                None => {
                    panic!("encode_and_send_data_to_teams: Building without width and height.");
                }
            }
        }

        game.units.kill_unit(death.id);

        for &wpn_id in &game.units.weapons(death.id).to_vec() {
            game.weapons.kill_weapon(wpn_id)
        }
    }

    game.logger.clear();

    if frame_number % 1 == 0 {
        for &team in &team_iter {
            let mut unit_msg = Cursor::new(Vec::new());
            let _ = unit_msg.write_u32::<BigEndian>(frame_number as u32);

            // CONVERT UNITS INTO DATA PACKETS
            for &id in &game.units.iter() {
                let unit_team = game.units.team(id);
                let unit_visible = game.teams.visible[team][id];

                if unit_team == team || unit_visible {
                    unit::encode(game, id, &mut unit_msg);
                }
            }

            let mut misl_msg = Cursor::new(Vec::new());
            let _ = misl_msg.write_u32::<BigEndian>(frame_number as u32);

            // CONVERT MISSILES INTO DATA PACKETS
            for &id in &game.missiles.iter() {
                if game.teams.visible_missiles[team][id] {
                    missile::encode(game, id, &mut misl_msg);
                }
            }

            let team_usize = unsafe {
                team.usize_unwrap()
            };

            let mut team_msg = Cursor::new(Vec::new());
            let _ = team_msg.write_u32::<BigEndian>(frame_number);
            let _ = team_msg.write_u8(4);
            let _ = team_msg.write_u8(team_usize as u8);
            let _ = team_msg.write_u32::<BigEndian>(game.teams.metal[team] as u32);
            let _ = team_msg.write_u32::<BigEndian>(game.teams.energy[team] as u32);

            netcom::send_message_to_team(game.netcom.clone(), team_msg.into_inner(), team_usize);
            netcom::send_message_to_team(game.netcom.clone(), misl_msg.into_inner(), team_usize);
            netcom::send_message_to_team(game.netcom.clone(), unit_msg.into_inner(), team_usize);
        }
    }
}