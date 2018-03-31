#![allow(dead_code)]
#![feature(plugin)]
#![feature(test)]

extern crate core;
extern crate time;
extern crate byteorder;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate enum_primitive;

mod data;
mod pathing;
mod libs;
mod behavior;
mod useful_bits;
mod setup_game;

use self::time::PreciseTime;
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io;
use std::time::Duration;
use std::thread::sleep;
use std::io::Cursor;
use libs::netcom;
use libs::tmx_decode::MapData;

use data::game::Game;
use data::logger;
use data::kdt_point as kdtp;
use data::aliases::*;
use setup_game::setup_game;

use behavior::missile::core as missile;
use behavior::unit::core as unit;
use behavior::weapon::core as weapon;

fn main() {
    //libs::fine_grid::bench_fine_grid();
    //libs::bitvec::los_visual();
    //bytegrid::test();
    //pathing::path_grid::bench();
    //pathing::path_grid::test();
    //libs::kdt::bench();
    //movement::test_circle_line_intersection();
    main_main();
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
    let players = [
        ("p1".to_string(), "p1".to_string(), 0),
        ("p2".to_string(), "p2".to_string(), 0),
        ("p3".to_string(), "p3".to_string(), 1),
        ("p4".to_string(), "p4".to_string(), 1),
    ];

    let netc = netcom::new(&players, &port, &address);

    let (units,unit_id_map,missiles,missile_id_map,encoded_unit_info,encoded_misl_info) = setup_game::list();

    let map_data = MapData::new("./maps/Map2.json");

    let game = &mut Game::new(4096, 8, map_data, units, unit_id_map, missiles, missile_id_map, encoded_unit_info, encoded_misl_info, netc);
    setup_game(game);

    println!("Game started.");
    let mut loop_count: u32 = 0;

    loop {
        let start_time = PreciseTime::now();
        let player_msgs = netcom::get_messages(&game.netcom);

        data::game::incorporate_messages(game, player_msgs);

        // RESET ECONOMY TRACKING
        for &team in &game.teams.iter() {
            game.teams.prime_output[team] = 0.0;
            game.teams.energy_output[team] = 0.0;
            game.teams.prime_drain[team] = 0.0;
            game.teams.energy_drain[team] = 0.0;
        }

        // STEP MISSILES
        for &id in &game.missiles.iter() {
            missile::step_missile(game, id);
        }

        // STEP UNITS
        let unit_iterator = game.units.iter();

        for &id in &unit_iterator {
            if game.units.progress(id) >= game.units.build_cost(id) {
                unit::event_handler(game, UnitEvent::UnitSteps(id));
            }
        }

        // MOVE, REGEN, & OUTPUT RESOURCES
        for &id in &unit_iterator {
            let team = game.units.team(id);
            if game.units.progress(id) >= game.units.build_cost(id) {
                unit::move_and_collide_and_correct(game, id);

                game.teams.prime_output[team] += game.units.prime_output(id);
                game.teams.energy_output[team] += game.units.energy_output(id);
                game.teams.prime[team] += game.units.prime_output(id);
                game.teams.energy[team] += game.units.energy_output(id);
                let health = game.units.health(id);
                let health_regen = game.units.health_regen(id);
                let max_health = game.units.max_health(id);
                game.units.set_health(
                    id,
                    f64::min(max_health, health + health_regen),
                );

                for i in 0..game.units.weapons(id).len() {
                    let mut wpn = game.units.weapons(id)[i].clone();
                    weapon::attack_orders(game, &mut wpn, id);
                    game.units.mut_weapons(id)[i] = wpn;
                }
            }
        }

        game.unit_kdt = kdtp::populate_with_kdtunits(&game);
        game.missile_kdt = kdtp::populate_with_kdtmissiles(&game.missiles);

        let frame_time = 1.0 / game.fps();
        for &team in &game.teams.iter() {
            // CLEAR VISIBLE UNITS
            for &id in &unit_iterator {
                game.teams.visible[team][id] = match game.teams.visible[team][id] {
                    Visibility::None => Visibility::None,
                    Visibility::Full(dur) => {
                        Visibility::Partial(dur - frame_time)
                    }
                    Visibility::Partial(dur) => {
                        if dur - frame_time <= 0.0 {
                            Visibility::None
                        }
                        else {
                            Visibility::Partial(dur - frame_time)
                        }
                    }
                    Visibility::RadarBlip(dur) => {
                        if dur - frame_time <= 0.0 {
                            Visibility::None
                        }
                        else {
                            Visibility::RadarBlip(dur - frame_time)
                        }
                    }
                };
            }

            // CLEAR VISIBLE MISSILES
            for &id in &game.missiles.iter() {
                game.teams.visible_missiles[team][id] = match game.teams.visible_missiles[team][id] {
                    Visibility::None => Visibility::None,
                    Visibility::Full(dur) => {
                        Visibility::Partial(dur - frame_time)
                    }
                    Visibility::Partial(dur) => {
                        if dur - frame_time <= 0.0 {
                            Visibility::None
                        }
                        else {
                            Visibility::Partial(dur - frame_time)
                        }
                    }
                    Visibility::RadarBlip(dur) => {
                        if dur - frame_time <= 0.0 {
                            Visibility::None
                        }
                        else {
                            Visibility::RadarBlip(dur - frame_time)
                        }
                    }
                };
            }

            // FIND VISIBLE UNITS AND MISSILES
            for &id in &unit_iterator {
                if game.units.team(id) == team {
                    let vis_enemies = kdtp::enemies_in_vision(game, id);
                    let sight_dur = game.units.sight_duration(id);

                    for kdtp in vis_enemies {
                        if let Some(id) = game.units.target_id(kdtp.target) {
                            game.teams.visible[team][id] = match game.teams.visible[team][id] {
                                Visibility::None => Visibility::Full(sight_dur),
                                Visibility::Full(dur) | Visibility::Partial(dur) | Visibility::RadarBlip(dur) => {
                                    if dur < sight_dur {
                                        Visibility::Full(sight_dur)
                                    }
                                    else {
                                        Visibility::Full(dur)
                                    }
                                }
                            };
                        }
                    }

                    let vis_missiles = unit::missiles_in_vision(game, id);

                    for kdtp in vis_missiles {
                        game.teams.visible_missiles[team][kdtp.id] = match game.teams.visible_missiles[team][kdtp.id] {
                                Visibility::None => Visibility::Full(sight_dur),
                                Visibility::Full(dur) | Visibility::Partial(dur) | Visibility::RadarBlip(dur) => {
                                    if dur < sight_dur {
                                        Visibility::Full(sight_dur)
                                    }
                                    else {
                                        Visibility::Full(dur)
                                    }
                                }
                            };
                    }
                }
            }
            // ADJUST TEAMS RESOURCES
            let build_power_distribution = game.teams.get_build_power_applications(team);
            let total_energy = game.teams.energy[team];
            let total_prime = game.teams.prime[team];
            let mut total_prime_drain = 0.0;
            let mut total_energy_drain = 0.0;
            let mut prime = game.teams.prime[team];
            let mut energy = game.teams.energy[team];

            for &(id, build_power) in &build_power_distribution {
                let build_cost = game.units.build_cost(id);
                let prime_cost = game.units.prime_cost(id);
                let energy_cost = game.units.energy_cost(id);
                let build_ratio = build_power / build_cost;

                total_prime_drain += prime_cost * build_ratio;
                total_energy_drain += energy_cost * build_ratio;
            }

            let energy_drain_ratio = f64::min(1.0, total_energy / total_energy_drain);
            let prime_drain_ratio = f64::min(1.0, total_prime / total_prime_drain);
            let drain_ratio = f64::min(energy_drain_ratio, prime_drain_ratio);

            for &(id, build_power) in &build_power_distribution {
                let prime_cost = game.units.prime_cost(id);
                let energy_cost = game.units.energy_cost(id);
                let build_fraction = if prime_cost > 0.0 && energy_cost > 0.0 {
                    build_power * drain_ratio
                } else {
                    build_power
                };
                let progress = game.units.progress(id);
                let build_cost = game.units.build_cost(id);
                let new_progress = progress + build_fraction;
                let health = game.units.health(id);
                let max_health = game.units.max_health(id);
                let new_health = health + max_health * (build_fraction / build_cost);

                if new_health > max_health {
                    let excess = (new_health - max_health) / max_health;
                    prime += excess * prime_cost;
                    energy += excess * energy_cost;
                    game.units.set_health(id, max_health);
                    game.units.set_progress(id, build_cost);
                }
                else {
                    game.units.set_health(id, new_health);
                    game.units.set_progress(id, new_progress);
                }
            }

            prime -= total_prime_drain * drain_ratio;
            energy -= total_energy_drain * drain_ratio;
            let max_prime = game.teams.max_prime[team];
            let max_energy = game.teams.max_energy[team];

            game.teams.prime_drain[team] = total_prime_drain;
            game.teams.energy_drain[team] = total_energy_drain;
            game.teams.prime[team] = f64::min(max_prime, prime);
            game.teams.energy[team] = f64::min(max_energy, energy);
        }
        game.frame_number = loop_count;
        encode_and_send_data_to_teams(game);

        // LOOP TIMING STUFF
        loop_count += 1;
        let end_time = PreciseTime::now();
        let time_spent = start_time.to(end_time).num_milliseconds();

        if (1000 / FPS as i64) - time_spent > 0 {
            sleep(Duration::from_millis(
                ((1000 / FPS as i64) - time_spent) as u64,
            ));
        } else {
            println!(
                "Logic is laggy. Loop# {}. Time (ms): {:?}",
                loop_count,
                time_spent
            );
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
        logger::encode_order_completed(game, team, &mut logg_msg);
        logger::encode_melee_smacks(game, team, &mut logg_msg);
        logger::encode_construction(game, team, &mut logg_msg);

        let team_usize = unsafe { team.usize_unwrap() };
        netcom::send_message_to_team(game.netcom.clone(), logg_msg.into_inner(), team_usize);
    }

    for &team in &team_iter {
        for ref boom in &game.logger.missile_booms {
            // NOTE! Sets exploded missiles visibility to false so they aren't encoded twice
            game.teams.visible_missiles[team][boom.id] = Visibility::None;
        }
        for &death in &game.logger.unit_deaths {
            // NOTE! Sets dead units visibility to false so they aren't encoded twice
            game.teams.visible[team][death.id] = Visibility::None;
        }
    }

    for ref boom in &game.logger.missile_booms {
        game.missiles.kill_missile(boom.id);
    }

    for &death in &game.logger.unit_deaths {
        let team = game.units.team(death.id);

        if game.units.is_structure(death.id) {
            match game.units.width_and_height(death.id) {
                Some((w, h)) => {
                    let (x, y) = game.units.xy(death.id);
                    let hw = w as f64 / 2.0;
                    let hh = h as f64 / 2.0;
                    let bx = (x - hw + 0.0001) as isize;
                    let by = (y - hh + 0.0001) as isize;

                    for xo in bx..bx + w {
                        for yo in by..by + h {
                            game.bytegrid.set_point(true, (xo, yo));
                            game.teams.jps_grid[team].open_point((xo, yo));
                        }
                    }
                }
                None => {
                    panic!("encode_and_send_data_to_teams: Building without width and height.");
                }
            }
        }
    }

    game.logger.clear();

    for &team in &team_iter {
        let mut unit_msg = Cursor::new(Vec::new());
        let _ = unit_msg.write_u32::<BigEndian>(frame_number as u32);

        // CONVERT UNITS INTO DATA PACKETS
        for &id in &game.units.iter() {
            let unit_team = game.units.team(id);

            if unit_team == team {
                unit::encode(game, id, &mut unit_msg);
            }
            else {
                match game.teams.visible[team][id] {
                    Visibility::None => (),
                    Visibility::Full(_) => {
                        unit::encode(game, id, &mut unit_msg);
                    }
                    Visibility::Partial(_) => {
                        unit::encode(game, id, &mut unit_msg);
                    }
                    Visibility::RadarBlip(_) => {
                        unit::encode(game, id, &mut unit_msg);
                    }
                }
            }
        }

        let mut misl_msg = Cursor::new(Vec::new());
        let _ = misl_msg.write_u32::<BigEndian>(frame_number as u32);

        // CONVERT MISSILES INTO DATA PACKETS
        for &id in &game.missiles.iter() {
            match game.teams.visible_missiles[team][id] {
                    Visibility::None => (),
                    Visibility::Full(_) => {
                        missile::encode(game, id, &mut misl_msg);
                    }
                    Visibility::Partial(_) => {
                        missile::encode(game, id, &mut misl_msg);
                    }
                    Visibility::RadarBlip(_) => {
                        missile::encode(game, id, &mut misl_msg);
                    }
                }
        }

        let team_usize = unsafe { team.usize_unwrap() };

        let mut team_msg = Cursor::new(Vec::new());
        let _ = team_msg.write_u32::<BigEndian>(frame_number);
        let _ = team_msg.write_u8(ClientMessage::TeamInfo as u8);
        let _ = team_msg.write_u8(team_usize as u8);
        let _ = team_msg.write_u32::<BigEndian>(game.teams.max_prime[team] as u32);
        let _ = team_msg.write_u32::<BigEndian>(game.teams.prime[team] as u32);
        let _ = team_msg.write_f64::<BigEndian>(game.teams.prime_output[team] * game.fps());
        let _ = team_msg.write_f64::<BigEndian>(game.teams.prime_drain[team] * game.fps());

        let _ = team_msg.write_u32::<BigEndian>(game.teams.max_energy[team] as u32);
        let _ = team_msg.write_u32::<BigEndian>(game.teams.energy[team] as u32);
        let _ = team_msg.write_f64::<BigEndian>(game.teams.energy_output[team] * game.fps());
        let _ = team_msg.write_f64::<BigEndian>(game.teams.energy_drain[team] * game.fps());

        netcom::send_message_to_team(game.netcom.clone(), team_msg.into_inner(), team_usize);
        netcom::send_message_to_team(game.netcom.clone(), misl_msg.into_inner(), team_usize);
        netcom::send_message_to_team(game.netcom.clone(), unit_msg.into_inner(), team_usize);
    }
}