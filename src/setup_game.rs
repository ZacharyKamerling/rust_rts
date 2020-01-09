extern crate rand;

use self::rand::Rng;
use data::game::Game;
use data::aliases::*;
use data::units::Unit;
use std::fs;
use std::io::prelude::*;
use std::io::Cursor;
use byteorder::{WriteBytesExt, BigEndian};

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();
    let fps = game.fps();

    if let Some(team) = game.teams.make_team() {
        game.teams.max_prime[team] = 1000.0;
        game.teams.max_energy[team] = 1000.0;
        game.teams.prime[team] = 1000.0;
        game.teams.energy[team] = 1000.0;

        for _ in 0..2000 {
            match game.units.make_from_name(fps, "Medium1".to_string()) {
                Some(id) => {
                    let x = rng.gen_range(0.0, 24.0);
                    let y = rng.gen_range(0.0, 96.0);
                    game.units.set_xy(id, (x, y));
                    game.units.set_team(id, team);
                    let prog_required = game.units.build_cost(id);
                    let max_health = game.units.max_health(id);
                    game.units.set_progress(id, prog_required);
                    game.units.set_health(id, max_health);
                }
                None => panic!("setup_game: Not enough unit IDs to go around."),
            }
        }
    }

    if let Some(team) = game.teams.make_team() {
        game.teams.max_prime[team] = 1000.0;
        game.teams.max_energy[team] = 1000.0;
        game.teams.prime[team] = 1000.0;
        game.teams.energy[team] = 1000.0;

        for _ in 0..2000 {
            match game.units.make_from_name(fps, "Medium1".to_string()) {
                Some(id) => {
                    let x = rng.gen_range(36.0, 108.0);
                    let y = rng.gen_range(0.0, 24.0);
                    game.units.set_xy(id, (x, y));
                    game.units.set_team(id, team);
                    let prog_required = game.units.build_cost(id);
                    let max_health = game.units.max_health(id);
                    game.units.set_progress(id, prog_required);
                    game.units.set_health(id, max_health);
                }
                None => panic!("setup_game: Not enough unit IDs to go around."),
            }
        }
    }

    let (width, height) = game.map_data.width_and_height();

    for team in game.teams.iter() {
        for i in 0..game.map_data.collisions().len() {
            let collision = game.map_data.collisions()[i];
            let x = i % width;
            let y = i / width;
            let xy = (x as isize, (height - y - 1) as isize);

            match collision {
                0 | 3 | 4 => {
                    game.teams.jps_grid[team].close_point(xy);
                }
                _ => (),
            }
        }
    }

    for i in 0..game.map_data.collisions().len() {
        let collision = game.map_data.collisions()[i];
        let x = i % width;
        let y = i / width;
        let xy = (x as isize, (height - y - 1) as isize);

        match collision {
            0 | 3 | 4 => {
                game.bytegrid.set_point(false, xy);
            }
            _ => (),
        }
    }
}

pub fn list() -> (VecUID<UnitTypeID, Unit>, UIDMapping<UnitTypeID>, VecUID<MissileTypeID, Missile>, UIDMapping<MissileTypeID>, Vec<u8>, Vec<u8>) {
    let mut unit_list = Vec::new();
    let mut misl_list = Vec::new();
    let mut unit_uids = UIDMapping::new(256);
    let mut misl_uids = UIDMapping::new(256);
    let mut unit_info = Cursor::new(Vec::new());
    let mut misl_info = Cursor::new(Vec::new());

    // Convert JSON to units & create unit info message for clients
    for entry in fs::read_dir("./src/units/").unwrap() {
        let mut file = fs::File::open(entry.unwrap().path()).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        if let Some(unit) = Unit::from_json(contents.as_ref()) {
            let bytes: Vec<u8> = contents.into_bytes();

            let _ = unit_info.write_u8(ClientMessage::UnitInfo as u8);
            let _ = unit_info.write_u32::<BigEndian>(bytes.len() as u32);

            for byte in bytes {
                let _ = unit_info.write_u8(byte);
            }

            unit_list.push(unit);
        }
        else {
            println!("Failed to load: {}", contents);
        }
    }

    // Convert JSON to missiles & create missile info message for clients
    for entry in fs::read_dir("./src/missiles/").unwrap() {
        let mut file = fs::File::open(entry.unwrap().path()).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        if let Some(misl) = Missile::from_json(contents.as_ref()) {
            let bytes: Vec<u8> = contents.into_bytes();

            let _ = misl_info.write_u8(ClientMessage::MissileInfo as u8);
            let _ = misl_info.write_u32::<BigEndian>(bytes.len() as u32);

            for byte in bytes {
                let _ = misl_info.write_u8(byte);
            }

            misl_list.push(misl);
        }
        else {
            println!("Failed to load: {}", contents);
        }
    }

    // Give each unit type an ID
    for proto in unit_list.iter_mut() {
        match unit_uids.name_to_id(proto.name().clone()) {
            Some(unit_type_id) => {
                proto.set_unit_type(Some(unit_type_id));
                println!("Unit Name: {:?}, ID: {:?}", proto.name(), unit_type_id);
            }
            None => {
                panic!("No more ids available");
            }
        }
    }

    // Give each missile type an ID
    for proto in misl_list.iter_mut() {
        match misl_uids.name_to_id(proto.name().clone()) {
            Some(misl_type_id) => {
                proto.set_missile_type_id(Some(misl_type_id));
                println!("Misl Name: {:?}, ID: {:?}", proto.name(), misl_type_id);
            }
            None => {
                panic!("No more ids available");
            }
        }
    }

    // Change references from names to IDs
    for unit in unit_list.iter_mut() {

        // Set each weapons missile ID
        for wpn in unit.mut_weapons().iter_mut() {
            match wpn.attack().clone() {
                Attack::Missile(Err(ref missile_name)) => {
                    match misl_uids.id(missile_name.clone()) {
                        Some(misl_type_id) => {
                            *wpn.mut_attack() = Attack::Missile(Ok(misl_type_id));
                        }
                        None => {
                            panic!("You have a bad missile reference for {}.", missile_name);
                        }
                    }
                }
                a => println!("How is this possible! {:?}", a),
            }
        }

        // Set build roster IDs
        for build_rostee in unit.build_roster_names().clone().iter() {
            match unit_uids.id(build_rostee.clone()) {
                Some(unit_type_id) => {
                    unit.mut_build_roster().insert(unit_type_id);
                }
                None => {
                    panic!("You have a bad build roster reference for {}.", build_rostee);
                }
            }
        }

        // Set train roster IDs
        for train_rostee in unit.train_roster_names().clone().iter() {
            match unit_uids.id(train_rostee.clone()) {
                Some(unit_type_id) => {
                    unit.mut_train_roster().insert(unit_type_id);
                }
                None => {
                    panic!("You have a bad train roster reference for {}.", train_rostee);
                }
            }
        }
    }

    // Convert Vectors to VecUIDs
    let mut unit_vec = VecUID::full_vec(256, Unit::new());
    for i in 0..unit_list.len() {
        let utid = unsafe { UnitTypeID::usize_wrap(i) };
        unit_vec[utid] = unit_list[i].clone();
    }

    let mut misl_vec = VecUID::full_vec(256, Missile::new());
    for i in 0..misl_list.len() {
        let utid = unsafe { MissileTypeID::usize_wrap(i) };
        misl_vec[utid] = misl_list[i].clone();
    }

    (unit_vec, unit_uids, misl_vec, misl_uids, unit_info.into_inner(), misl_info.into_inner())
}