// Keeps track of important events that need to be sent to clients.

extern crate byteorder;

use data::aliases::*;
use data::game::{Game};
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;

#[derive(Clone,Copy)]
pub struct MissileBoom {
    pub id:             MissileID,
    pub missile_type:   MissileTypeID,
    pub x:              f32,
    pub y:              f32,
}

#[derive(Clone,Copy)]
pub struct MeleeSmack {
    pub id:     UnitID,
}

#[derive(Clone,Copy)]
pub struct UnitDeath {
    pub id:             UnitID,
    pub damage_type:    DamageType,
}

pub struct Logger {
    pub unit_deaths:        Vec<UnitDeath>,
    pub missile_booms:      Vec<MissileBoom>,
    pub melee_smacks:       Vec<MeleeSmack>,
}

impl Logger {

    pub fn new() -> Logger {
        Logger {
            unit_deaths:        Vec::new(),
            missile_booms:      Vec::new(),
            melee_smacks:       Vec::new(),
        }
    }

    pub fn log_missile_boom(&mut self, missile_type: MissileTypeID, m_id: MissileID, (x,y): (f32,f32)) {
        let boom = MissileBoom {
            id: m_id,
            missile_type: missile_type,
            x: x,
            y: y,
        };
        self.missile_booms.push(boom);
    }

    pub fn clear(&mut self) {
        self.unit_deaths.clear();
        self.missile_booms.clear();
        self.melee_smacks.clear();
    }
}

pub fn encode_missile_booms(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for &boom in &game.logger.missile_booms {
        if game.teams.visible_missiles[team][boom.id] {
            let _ = vec.write_u8(2);
            let _ = vec.write_u8(boom.missile_type as u8);
            let _ = vec.write_u16::<BigEndian>(unsafe { boom.id.usize_unwrap() as u16 });
            let _ = vec.write_u16::<BigEndian>((boom.x * 64.0) as u16);
            let _ = vec.write_u16::<BigEndian>((boom.y * 64.0) as u16);
        }
    }
}

pub fn encode_unit_deaths(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for &death in &game.logger.unit_deaths.to_vec() {
        if game.teams.visible[team][death.id] {
            let _ = vec.write_u8(3);
            unsafe {
                let _ = vec.write_u16::<BigEndian>(death.id.usize_unwrap() as u16);
            }
            let _ = vec.write_u8(death.damage_type as u8);
        }
    }
}

pub fn encode_melee_smacks(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for &smack in &game.logger.melee_smacks {
        if game.teams.visible[team][smack.id] {
            let _ = vec.write_u8(4);
            unsafe {
                let _ = vec.write_u16::<BigEndian>(smack.id.usize_unwrap() as u16);
            }
        }
    }
}