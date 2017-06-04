extern crate rand;
extern crate byteorder;
extern crate websocket;
extern crate num;

use libs::kdt::{KDTree};
use libs::bytegrid::{ByteGrid};
use libs::netcom::{Netcom,send_message_to_player};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use self::num::FromPrimitive;
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
    encoded_map_data:               Vec<u8>,
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
              , encoded_map_data: Vec<u8>
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
            encoded_map_data: encoded_map_data,
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
        if let Ok(msg_num) = cursor.read_u8() {
            if let Some(msg_type) = ServerMessage::from_u8(msg_num) {
                unsafe {
                    match msg_type {
                        ServerMessage::Move => {
                            read_move_message(game, TeamID::usize_wrap(team), cursor);
                        }
                        ServerMessage::AttackTarget => {
                            read_attack_target_message(game, TeamID::usize_wrap(team), cursor);
                        }
                        ServerMessage::Build => {
                            read_build_message(game, TeamID::usize_wrap(team), cursor)
                        }
                        ServerMessage::AttackMove => {
                            read_attack_move_message(game, TeamID::usize_wrap(team), cursor);
                        }
                        ServerMessage::MapInfoRequest => {
                            send_tilegrid_info(game, name);
                        }
                    }
                }
            }
            else {
                println!("Received poorly formatted message from {}.", name);
            }
        }
    }
}

fn send_tilegrid_info(game: &Game, name: String) {
    // We add 5 bytes to the encoded data for the frame number and message tag
    let mut msg = Cursor::new(Vec::with_capacity(game.encoded_map_data.len() + 5));
    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let _ = msg.write_u8(ClientMessage::MapInfo as u8);
    let mut vec = msg.into_inner();
    vec.append(&mut game.encoded_map_data.clone());

    send_message_to_player(game.netcom.clone(), vec, name);
}

fn read_move_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord_id = vec.read_u32::<BigEndian>();
    let res_ord = vec.read_u8();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord_id), Ok(ord_num), Ok(x), Ok(y)) = (res_ord_id, res_ord, res_x, res_y) {
        let order_id = unsafe {
            OrderID::usize_wrap(ord_id as usize)
        };
        let order_type = OrderType::Move(MoveGroup::new((x as f64, y as f64)));
        let move_order = Rc::new(Order {order_type: order_type, order_id: order_id});

        while let Ok(uid) = vec.read_u16::<BigEndian>() {
            let id = unsafe {
                UnitID::usize_wrap(uid as usize)
            };
            if (uid as usize) < game.max_units &&
                game.units.team(id) == team &&
                !game.units.is_automatic(id) &&
                !game.units.is_structure(id)
            {
                if let Some(ord) = QueueOrder::from_u8(ord_num) {
                    match ord {
                        QueueOrder::Replace => { // REPLACE
                            game.units.mut_orders(id).clear();
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        QueueOrder::Append => { // APPEND
                            game.units.mut_orders(id).push_back(move_order.clone());
                        }
                        QueueOrder::Prepend => { // PREPEND
                            game.units.mut_orders(id).push_front(move_order.clone());
                        }
                    }
                }
            }
        }
    }
}

fn read_build_message(game: &mut Game, team: TeamID, vec: &mut Cursor<Vec<u8>>) {
    let res_ord_id = vec.read_u32::<BigEndian>();
    let res_ord = vec.read_u8();
    let res_type = vec.read_u16::<BigEndian>();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord_id), Ok(ord), Ok(x64), Ok(y64), Ok(bld_type)) = (res_ord_id, res_ord, res_x, res_y, res_type) {
        let build_type = unsafe {
            UnitTypeID::usize_wrap(bld_type as usize)
        };
        let order_id = unsafe {
            OrderID::usize_wrap(ord_id as usize)
        };
        let order_type = OrderType::Build(BuildGroup::new(build_type, BuildTarget::Point((x64 as f64, y64 as f64))));
        let build_order = Rc::new(Order {order_type: order_type, order_id: order_id});

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
    let res_ord_id = vec.read_u32::<BigEndian>();
    let res_ord = vec.read_u8();
    let res_x = vec.read_f64::<BigEndian>();
    let res_y = vec.read_f64::<BigEndian>();

    if let (Ok(ord_id), Ok(ord), Ok(x), Ok(y)) = (res_ord_id, res_ord, res_x, res_y) {
        let order_id = unsafe {
            OrderID::usize_wrap(ord_id as usize)
        };
        let order_type = OrderType::AttackMove(MoveGroup::new((x as f64, y as f64)));
        let move_order = Rc::new(Order {order_type: order_type, order_id: order_id});

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
    let res_ord_id = vec.read_u32::<BigEndian>();
    let res_ord = vec.read_u8();
    let res_tid = vec.read_u16::<BigEndian>(); // Target ID

    if let (Ok(ord_id), Ok(ord), Ok(tid)) = (res_ord_id, res_ord, res_tid) {
        let order_id = unsafe {
            OrderID::usize_wrap(ord_id as usize)
        };
        let t_id = unsafe {
            UnitID::usize_wrap(tid as usize)
        };
        let (x,y) = game.units.xy(t_id);
        let unit_target = UnitTarget::new(&game.units, t_id);
        let order_type = OrderType::AttackTarget(MoveGroup::new((x as f64, y as f64)), unit_target);
        let new_order = Rc::new(Order {order_type: order_type, order_id: order_id});

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