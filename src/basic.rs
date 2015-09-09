extern crate byteorder;

use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::io::Cursor;
use data::*;
use movement::denormalize;
use std::f32::consts::{PI};

fn serialize(vec: &mut Cursor<Vec<u8>>) {
    let _ = vec.write_u16::<BigEndian>(0);
    let _ = vec.read_u16::<BigEndian>();
}

fn encode(game: Game, id: UnitID) -> Cursor<Vec<u8>> {
    let mut vec = Cursor::new(Vec::with_capacity(16));
    let units = game.units;
    let f = denormalize(units.facing[id]);

    let _ = vec.write_u8(0);
    let _ = vec.write_u8(units.unit_type[id] as u8);
    let _ = vec.write_u16::<BigEndian>(id as u16);
    let _ = vec.write_u16::<BigEndian>((units.x[id] * 64.0) as u16);
    let _ = vec.write_u16::<BigEndian>((units.y[id] * 64.0) as u16);
    let _ = vec.write_u8(units.anim[id] as u8);
    let _ = vec.write_u8(units.team[id] as u8);
    let _ = vec.write_u8((f * 255.0 / (2.0 * PI)) as u8);
    let _ = vec.write_u8((units.health[id] / units.max_health[id] * 255.0) as u8);
    let _ = vec.write_u8((units.progress[id] / units.progress_required[id] * 255.0) as u8);
    /*
    for wpn_id in 0..units.weapons[id].facing.len() {
        let wpn = units.weapons[id];
        let _ = vec.write_u8((wpn.facing[wpn_id] * 255.0 / (2.0 * PI)) as u8);
        let _ = vec.write_u8((wpn.anim[wpn_id] as u8));
    }
    */

    let ref passengers = units.passengers[id];
    let _ = vec.write_u8((passengers.len() as u8));

    for psngr in passengers.iter() {
        let _ = vec.write_u16::<BigEndian>(*psngr as u16);
    }
    vec
}

fn decode() {

}

/*
Type        = 16
ID          = 16
X           = 16
Y           = 16
Anim        = 8
Owner       = 8
Facing      = 8
Health      = 8
Progress    = 8
wpn_facings = 8 * num_weapons_on_unit_type
wpn_anims   = 8 * i
j_riders    = 8
rider_IDs   = 16 * N
*/