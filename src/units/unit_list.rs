use units;
use data::units::Unit;
use data::aliases::*;
use std::fs;
use std::io::prelude::*;
use std::io::Cursor;
use byteorder::{WriteBytesExt, BigEndian};

pub fn list() -> (VecUID<UnitTypeID, Unit>, UIDMapping<UnitTypeID>, VecUID<MissileTypeID, Missile>, UIDMapping<MissileTypeID>, Vec<u8>) {
    let mut unit_list = Vec::new();
    let mut misl_list = Vec::new();
    let mut unit_uids = UIDMapping::new(256);
    let mut misl_uids = UIDMapping::new(256);
    let mut unit_info = Cursor::new(Vec::new());

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
    }

    unit_list.push(units::extractor1::prototype());
    misl_list.push(units::fast1::missile_proto());
    misl_list.push(units::medium1::missile_proto());
    misl_list.push(units::artillery1::missile_proto());

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

    for unit in unit_list.iter_mut() {
        for wpn in unit.mut_weapons().iter_mut() {
            match wpn.attack().clone() {
                Attack::Missile(Err(ref missile_name)) => {
                    match misl_uids.name_to_id(missile_name.clone()) {
                        Some(misl_type_id) => {
                            *wpn.mut_attack() = Attack::Missile(Ok(misl_type_id));
                            println!("Wpn Name: {:?}, Misl ID: {:?}, Misl Name: {:?}", wpn.name(), misl_type_id, missile_name);
                        }
                        None => {
                            panic!("No more ids available");
                        }
                    }
                }
                a => println!("How is this possible! {:?}", a),
            }
        }
    }

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

    (unit_vec, unit_uids, misl_vec, misl_uids, unit_info.into_inner())
}