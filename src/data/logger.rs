// Keeps track of important events that need to be sent to clients.

extern crate byteorder;

use data::aliases::*;
use data::game::Game;
use data::units::UnitTarget;
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;

#[derive(Clone, Copy)]
pub struct MissileBoom {
    pub id: MissileID,
    pub missile_type: MissileTypeID,
    pub team: TeamID,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct MeleeSmack {
    id: UnitID,
}

#[derive(Clone, Copy)]
pub struct UnitDeath {
    pub id: UnitID,
    damage_type: DamageType,
}

#[derive(Clone, Copy)]
pub struct OrderCompleted {
    unit_target: UnitTarget,
    order_id: OrderID,
}

pub struct Logger {
    pub unit_deaths: Vec<UnitDeath>,
    pub missile_booms: Vec<MissileBoom>,
    melee_smacks: Vec<MeleeSmack>,
    orders_completed: Vec<OrderCompleted>,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            unit_deaths: Vec::new(),
            missile_booms: Vec::new(),
            melee_smacks: Vec::new(),
            orders_completed: Vec::new(),
        }
    }

    pub fn log_order_completed(&mut self, unit: UnitTarget, order_id: OrderID) {
        let completed = OrderCompleted {
            unit_target: unit,
            order_id: order_id,
        };
        self.orders_completed.push(completed);
    }

    pub fn log_missile_boom(&mut self, missile_type: MissileTypeID, m_id: MissileID, team: TeamID, (x, y): (f64, f64)) {
        let boom = MissileBoom {
            id: m_id,
            missile_type: missile_type,
            team: team,
            x: x,
            y: y,
        };
        self.missile_booms.push(boom);
    }

    pub fn log_unit_death(&mut self, id: UnitID, damage_type: DamageType) {
        let death = UnitDeath {
            id: id,
            damage_type: damage_type,
        };
        self.unit_deaths.push(death);
    }

    pub fn clear(&mut self) {
        self.unit_deaths.clear();
        self.missile_booms.clear();
        self.melee_smacks.clear();
        self.orders_completed.clear();
    }
}

pub fn encode_order_completed(game: &Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for completed in &game.logger.orders_completed {
        if let Some(unit_id) = completed.unit_target.id(&game.units) {
            if game.units.team(unit_id) == team {
                let _ = vec.write_u8(ClientMessage::OrderCompleted as u8);
                unsafe {
                    let _ = vec.write_u16::<BigEndian>(unit_id.usize_unwrap() as u16);
                    let _ = vec.write_u16::<BigEndian>(completed.order_id.usize_unwrap() as u16);
                }
            }
        }
    }
}

pub fn encode_missile_booms(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for &boom in &game.logger.missile_booms {
        if game.teams.visible_missiles[team][boom.id] {
            let _ = vec.write_u8(ClientMessage::MissileExplode as u8);
            unsafe {
                let _ = vec.write_u8(MissileTypeID::usize_unwrap(boom.missile_type) as u8);
                let _ = vec.write_u16::<BigEndian>(boom.id.usize_unwrap() as u16);
                let _ = vec.write_u16::<BigEndian>((boom.x * 64.0) as u16);
                let _ = vec.write_u16::<BigEndian>((boom.y * 64.0) as u16);
                let _ = vec.write_u8(boom.team.usize_unwrap() as u8);
            }
        }
    }
}

pub fn encode_unit_deaths(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    for &death in &game.logger.unit_deaths.to_vec() {
        if game.teams.visible[team][death.id] {
            let _ = vec.write_u8(ClientMessage::UnitDeath as u8);
            unsafe {
                let _ = vec.write_u16::<BigEndian>(death.id.usize_unwrap() as u16);
            }
            let _ = vec.write_u8(death.damage_type as u8);
        }
    }
}

/*
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
*/
