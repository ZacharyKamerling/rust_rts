extern crate rand;
extern crate byteorder;

use kdt::{KDTree};
use bytegrid::{ByteGrid};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::io::Cursor;
use data::logger::{Logger};
use data::units::{Units,ProtoUnit,UnitTarget};
use data::kdt_point::{KDTUnit,KDTMissile};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};
use data::move_groups::{MoveGroup};
use data::build_groups::{BuildGroup,BuildTarget};
use std::cell::Cell;
use std::rc::Rc;
use data::aliases::*;

pub struct Game {
    max_units:                      usize,
    max_weapons:                    usize,
    max_missiles:                   usize,
    rng:                            ThreadRng,
    random_offset_gen:              Range<f32>,
    pub unit_blueprints:            Vec<ProtoUnit>,
    pub weapon_blueprints:          Vec<Weapon>,
    pub missile_blueprints:         Vec<Missile>,
    pub units:                      Units,
    pub weapons:                    Weapons,
    pub missiles:                   Missiles,
    pub teams:                      Teams,
    pub unit_kdt:                   KDTree<KDTUnit>,
    pub missile_kdt:                KDTree<KDTMissile>,
    pub bytegrid:                   ByteGrid,
    pub logger:                     Logger,
}

impl Game {
    pub fn new(max_units: usize, max_teams: usize, width: usize, height: usize
              , unit_prototypes: Vec<ProtoUnit>
              , weapon_prototypes: Vec<Weapon>
              , missile_prototypes: Vec<Missile>
              ) -> Game {
        Game {
            max_units: max_units,
            max_weapons: max_units * 2,
            max_missiles: max_units * 4,
            rng: rand::thread_rng(),
            random_offset_gen: Range::new(-0.0001, 0.0001),
            unit_blueprints: Vec::new(),
            weapon_blueprints: Vec::new(),
            missile_blueprints: Vec::new(),
            units: Units::new(max_units, unit_prototypes),
            weapons: Weapons::new(max_units * 2, weapon_prototypes),
            missiles: Missiles::new(max_units * 4, missile_prototypes),
            teams: Teams::new(max_units, max_teams, width, height),
            unit_kdt: KDTree::new(Vec::new()),
            missile_kdt: KDTree::new(Vec::new()),
            bytegrid: ByteGrid::new(width as isize, height as isize),
            logger: Logger::new(),
        }
    }

    pub fn max_units(&self) -> usize { self.max_units }
    pub fn max_weapons(&self) -> usize { self.max_weapons }
    pub fn max_missiles(&self) -> usize { self.max_missiles }

    // Produces a tiny random offset.
    // This is useful to avoid units occupying the same spot and being unable to collide correctly.
    pub fn get_random_offset(&mut self) -> f32 {
        self.random_offset_gen.sample(&mut self.rng)
    }
}

pub fn incorporate_messages(game: &mut Game, msgs: Vec<(String, usize, Vec<u8>)>) {
    for msg in msgs {
        let (name,team,data) = msg;
        let cursor = &mut Cursor::new(data);
        let msg_type = cursor.read_u8();

        unsafe {
            match msg_type {
                Ok(0) => { // MOVE
                    read_move_message(game, TeamID::usize_wrap(team), cursor);
                }
                Ok(1) => { // ATTACK TARGET
                    read_attack_target_message(game, TeamID::usize_wrap(team), cursor);
                }
                Ok(2) => { // BUILD
                    read_build_message(game, TeamID::usize_wrap(team), cursor)
                }
                Ok(3) => { // ATTACK MOVE
                    read_attack_move_message(game, TeamID::usize_wrap(team), cursor);
                }
                _ => {
                    println!("Received poorly formatted message from {}.", name);
                }
            }
        }
    }
}

fn read_move_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_len = vec.read_u16::<BigEndian>();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    match (res_ord, res_x, res_y, res_len) {
        (Ok(ord), Ok(x), Ok(y), Ok(len)) => {
            let move_order = Rc::new(Order::Move(MoveGroup::new((x as f32, y as f32))));

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !game.units.is_structure(id)
                {
                    match ord {
                        0 => { // REPLACE
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(move_order.clone());
                        }
                        _ => ()
                    }
                }
            }
        }
        _ => ()
    }
}

fn read_build_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_len = vec.read_u16::<BigEndian>();
    let res_type = vec.read_u16::<BigEndian>();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    match (res_ord, res_len, res_x, res_y, res_type) {
        (Ok(ord), Ok(len), Ok(x64), Ok(y64), Ok(bld_type)) => {
            let build_order = Rc::new(Order::Build(BuildGroup::new(bld_type as usize, BuildTarget::Point((x64 as f32, y64 as f32)))));

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !game.units.is_structure(id)
                {
                    match ord {
                        0 => { // REPLACE
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(build_order.clone());
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(build_order.clone());
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(build_order.clone());
                        }
                        _ => ()
                    }
                }
            }
        }
        _ => ()
    }
}

fn read_attack_move_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_len = vec.read_u16::<BigEndian>();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    match (res_ord, res_x, res_y, res_len) {
        (Ok(ord), Ok(x), Ok(y), Ok(len)) => {
            let move_order = Rc::new(Order::AttackMove(MoveGroup::new((x as f32, y as f32))));

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !game.units.is_structure(id)
                {
                    match ord {
                        0 => { // REPLACE
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(move_order.clone());
                        }
                        _ => ()
                    }
                }
            }
        }
        _ => ()
    }
}

fn read_attack_target_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_tid = vec.read_u16::<BigEndian>(); // Target ID

    match (res_ord, res_tid) {
        (Ok(ord), Ok(tid)) => {
            let t_id = unsafe {
                UnitID::usize_wrap(tid as usize)
            };
            let (x,y) = game.units.xy(t_id);

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                let unit_target = UnitTarget::new(&game.units, t_id);
                let new_order = Rc::new(Order::AttackTarget(MoveGroup::new((x as f32, y as f32)), unit_target));

                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !game.units.is_structure(id)
                {
                    match ord {
                        0 => { // REPLACE
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(new_order.clone());
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(new_order.clone());
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(new_order.clone());
                        }
                        _ => ()
                    }
                }
            }
        }
        _ => ()
    }
}