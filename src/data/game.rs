extern crate rand;
extern crate byteorder;
extern crate websocket;

use libs::kdt::{KDTree};
use libs::bytegrid::{ByteGrid};
use libs::netcom::{Netcom,send_message_to_player};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use std::sync::{Arc, Mutex};
use std::io::Cursor;
use data::logger::{Logger};
use data::units::{Units,ProtoUnit,UnitTarget};
use data::kdt_point::{KDTUnit,KDTMissile};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};
use data::move_groups::{MoveGroup};
use data::build_groups::{BuildGroup,BuildTarget};
use std::rc::Rc;
use data::aliases::*;

pub struct Game {
    max_units:                      usize,
    max_weapons:                    usize,
    max_missiles:                   usize,
    rng:                            ThreadRng,
    random_offset_gen:              Range<f64>,
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
    pub netcom:                     Arc<Mutex<Netcom>>,
    pub frame_number:               u32,
}

impl Game {
    pub fn new(max_units: usize, max_teams: usize, (width,height): (usize,usize)
              , unit_prototypes: VecUID<UnitTypeID,ProtoUnit>
              , weapon_prototypes: Vec<Weapon>
              , missile_prototypes: Vec<Missile>
              , netcom: Arc<Mutex<Netcom>>
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
            netcom: netcom,
            frame_number: 0,
        }
    }

    pub fn max_units(&self) -> usize { self.max_units }
    pub fn max_weapons(&self) -> usize { self.max_weapons }
    pub fn max_missiles(&self) -> usize { self.max_missiles }

    // Produces a tiny random offset.
    // This is useful to avoid units occupying the same spot and being unable to collide correctly.
    pub fn get_random_offset(&mut self) -> f64 {
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
                Ok(4) => { // TILEGRID INFORMATION REQUEST
                    send_tilegrid_info(game, TeamID::usize_wrap(team), name);
                }
                _ => {
                    println!("Received poorly formatted message from {}.", name);
                }
            }
        }
    }
}

fn send_tilegrid_info(game: &Game, team: TeamID, name: String) {
    let grid = &game.teams.jps_grid[team];
    let (w,h) = grid.width_and_height();
    let mut msg = Cursor::new(Vec::new());

    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let _ = msg.write_u8(ClientMessage::MapInfo as u8);
    let _ = msg.write_u16::<BigEndian>(w as u16);
    let _ = msg.write_u16::<BigEndian>(h as u16);

    for y in 0..h {
        for x in 0..w {
            let state = grid.is_open((x,y));
            let _ = msg.write_u8(if state { 1 } else { 0 });
            let _ = msg.write_u8(0);
            let _ = msg.write_u8(0);
        }
    }

    send_message_to_player(game.netcom.clone(), msg.into_inner(), name);
}

fn read_move_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord), Ok(x), Ok(y)) = (res_ord, res_x, res_y) {
        let move_order = Rc::new(Order::Move(MoveGroup::new((x as f64, y as f64))));

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
}

fn read_build_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_type = vec.read_u16::<BigEndian>();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord), Ok(x64), Ok(y64), Ok(bld_type)) = (res_ord, res_x, res_y, res_type) {
        let build_type = unsafe {
            UnitTypeID::usize_wrap(bld_type as usize)
        };
        let build_order = Rc::new(Order::Build(BuildGroup::new(build_type, BuildTarget::Point((x64 as f64, y64 as f64)))));

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
}

fn read_attack_move_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord), Ok(x), Ok(y)) = (res_ord, res_x, res_y) {
        let move_order = Rc::new(Order::AttackMove(MoveGroup::new((x as f64, y as f64))));

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
}

fn read_attack_target_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord = vec.read_u8();
    let res_tid = vec.read_u16::<BigEndian>(); // Target ID

    if let (Ok(ord), Ok(tid)) = (res_ord, res_tid) {
        let t_id = unsafe {
            UnitID::usize_wrap(tid as usize)
        };
        let (x,y) = game.units.xy(t_id);

        while let Ok(uid) = vec.read_u16::<BigEndian>() {
            let id = unsafe {
                UnitID::usize_wrap(uid as usize)
            };
            let unit_target = UnitTarget::new(&game.units, t_id);
            let new_order = Rc::new(Order::AttackTarget(MoveGroup::new((x as f64, y as f64)), unit_target));

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
}