extern crate rand;
extern crate byteorder;

use kdt::{KDTree};
use bytegrid::{ByteGrid};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::io::Cursor;
use data::logger::{Logger};
use data::units::{Units,ProtoUnit};
use data::kdt_point::{KDTUnit,KDTMissile};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};
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

    pub fn clear_units_move_groups(&mut self, id: UnitID) {
        for i in 0..self.units.orders(id).len() {
            let ord = self.units.orders(id)[i];

            let opt_mg_id = match ord {
                Order::Move(id) => Some(id),
                Order::AttackMove(id) => Some(id),
                Order::AttackTarget(_) => None,
                Order::Build(_,_) => None,
            };

            match opt_mg_id {
                Some(mg_id) => {
                    self.units.move_groups.not_in_move_group_anymore(mg_id);
                }
                None => ()
            }
        }
    }
}

pub fn incorporate_messages(game: &mut Game, msgs: Vec<(String, usize, Vec<u8>)>) {
        for msg in msgs {
            let (name,team,data) = msg;
            let mut cursor = Cursor::new(data);
            let msg_type = cursor.read_u8();

            match msg_type {
                Ok(0) => { // MOVE COMMAND
                    unsafe {
                        read_move_message(game, TeamID::usize_wrap(team), &mut cursor);
                    }
                }
                Ok(1) => { // BUILD COMMAND
                    unsafe {
                        read_build_message(game, TeamID::usize_wrap(team), &mut cursor)
                    }
                }
                _ => {
                    println!("Received poorly formatted message from {}.", name);
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
            let mg_id = game.units.move_groups.make_group(len as usize, x as f32, y as f32);

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !(game.units.target_type(id) == TargetType::Structure)
                {
                    match ord {
                        0 => { // REPLACE
                            game.clear_units_move_groups(id);
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(Order::Move(mg_id));
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(Order::Move(mg_id));
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(Order::Move(mg_id));
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
    let res_type = vec.read_u16::<BigEndian>();
    let res_x = vec.read_i64::<BigEndian>();
    let res_y = vec.read_i64::<BigEndian>();

    match (res_ord, res_x, res_y, res_len, res_type) {
        (Ok(ord), Ok(x64), Ok(y64), Ok(structure_type)) => {
            let x = x64 as isize;
            let y = y64 as isize;

            while let Ok(uid) = vec.read_u16::<BigEndian>() {
                let id = unsafe {
                    UnitID::usize_wrap(uid as usize)
                };
                if (uid as usize) < game.max_units &&
                    game.units.team(id) == team &&
                    !game.units.is_automatic(id) &&
                    !(game.units.target_type(id) == TargetType::Structure)
                {
                    match ord {
                        0 => { // REPLACE
                            game.clear_units_move_groups(id);
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(Order::Build(structure_type as usize, (x,y)));
                        }
                        1 => { // APPEND
                            game.units.mut_orders(id).push_back(Order::Build(structure_type as usize, (x,y)));
                        }
                        2 => { // PREPEND
                            game.units.mut_orders(id).push_front(Order::Build(structure_type as usize, (x,y)));
                        }
                        _ => ()
                    }
                }
            }
        }
        _ => ()
    }
}