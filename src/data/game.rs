extern crate rand;
extern crate byteorder;
extern crate websocket;
extern crate num;

use libs::kdt::KDTree;
use libs::bytegrid::ByteGrid;
use libs::netcom::{Netcom, send_message_to_player};
use libs::tmx_decode::MapData;
use self::rand::distributions::{Sample, Range};
use self::rand::ThreadRng;
use self::byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use self::num::FromPrimitive;
use std::sync::{Arc, Mutex};
use std::io::Cursor;
use std::io;
use data::logger::Logger;
use data::units::{Units, Unit, UnitTarget};
use data::kdt_point::{KDTUnit, KDTMissile};
use data::teams::Teams;
use data::weapons::{Weapon};
use data::missiles::{Missiles, ProtoMissile};
use data::move_groups::MoveGroup;
use data::build_groups::{BuildGroup, BuildTarget};
use std::rc::Rc;
use data::aliases::*;

pub struct Game {
    pub fps: f64,
    max_units: usize,
    max_weapons: usize,
    max_missiles: usize,
    rng: ThreadRng,
    random_offset_gen: Range<f64>,
    encoded_map_data: Vec<u8>,
    pub map_data: MapData,
    pub units: Units,
    pub missiles: Missiles,
    pub teams: Teams,
    pub unit_kdt: KDTree<KDTUnit>,
    pub missile_kdt: KDTree<KDTMissile>,
    pub bytegrid: ByteGrid,
    pub logger: Logger,
    pub netcom: Arc<Mutex<Netcom>>,
    pub frame_number: u32,
}

impl Game {
    pub fn new(
        max_units: usize,
        max_teams: usize,
        map_data: MapData,
        unit_prototypes: VecUID<UnitTypeID, Unit>,
        missile_prototypes: VecUID<MissileTypeID, ProtoMissile>,
        netcom: Arc<Mutex<Netcom>>,
    ) -> Game {
        let (width, height) = map_data.width_and_height();

        Game {
            fps: 10.0,
            max_units: max_units,
            max_weapons: max_units * 2,
            max_missiles: max_units * 4,
            rng: rand::thread_rng(),
            random_offset_gen: Range::new(-0.0001, 0.0001),
            encoded_map_data: map_data.encode(),
            map_data: map_data,
            units: Units::new(max_units, unit_prototypes),
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

    pub fn max_units(&self) -> usize {
        self.max_units
    }
    pub fn max_weapons(&self) -> usize {
        self.max_weapons
    }
    pub fn max_missiles(&self) -> usize {
        self.max_missiles
    }

    // Produces a tiny random offset.
    // This is useful to avoid units occupying the same spot and being unable to collide correctly.
    pub fn get_random_offset(&mut self) -> f64 {
        self.random_offset_gen.sample(&mut self.rng)
    }
}

pub fn incorporate_messages(game: &mut Game, msgs: Vec<(String, usize, Vec<u8>)>) {
    for msg in msgs {
        let (name, team, data) = msg;
        let bytes = &mut Cursor::new(data);

        if let Ok(Some(msg_type)) = bytes.read_u8().map(ServerMessage::from_u8) {
            if let Ok(order_id) = unsafe {
                bytes.read_u32::<BigEndian>().map(|a| {
                    OrderID::usize_wrap(a as usize)
                })
            }
            {
                let team_id = unsafe { TeamID::usize_wrap(team) };

                match msg_type {
                    ServerMessage::Move => {
                        let _ = read_move_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::AttackTarget => {
                        //read_attack_target_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::Build => {
                        let _ = read_build_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::AttackMove => {
                        let _ = read_attack_move_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::MapInfoRequest => {
                        send_tilegrid_info(game, name);
                    }
                }
            }
        }
    }
}

fn add_order_to_units(game: &mut Game, team_id: TeamID, order: Rc<Order>, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();

    while let Ok(uid) = bytes.read_u16::<BigEndian>() {
        let unit_id = unsafe { UnitID::usize_wrap(uid as usize) };

        if (uid as usize) < game.max_units && game.units.team(unit_id) == team_id && !game.units.is_automatic(unit_id) &&
            !game.units.is_structure(unit_id)
        {
            match queue_order {
                QueueOrder::Replace => {
                    // REPLACE
                    game.units.mut_orders(unit_id).clear();
                    game.units.mut_orders(unit_id).push_back(order.clone());
                }
                QueueOrder::Append => {
                    // APPEND
                    game.units.mut_orders(unit_id).push_back(order.clone());
                }
                QueueOrder::Prepend => {
                    // PREPEND
                    game.units.mut_orders(unit_id).push_front(order.clone());
                }
            }
        }
    }

    Ok(())
}

fn send_tilegrid_info(game: &Game, name: String) {
    // We add 5 bytes to the encoded data for the frame number and message tag
    let len = game.encoded_map_data.len() + 5;
    let mut msg = Cursor::new(Vec::with_capacity(len));
    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let _ = msg.write_u8(ClientMessage::MapInfo as u8);
    let mut bytes = msg.into_inner();
    bytes.append(&mut game.encoded_map_data.clone());

    send_message_to_player(game.netcom.clone(), bytes, name);
}

fn read_move_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let x = bytes.read_f64::<BigEndian>()?;
    let y = bytes.read_f64::<BigEndian>()?;

    let order_type = OrderType::Move(MoveGroup::new((x as f64, y as f64)));
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, bytes)
}

fn read_attack_move_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let x = bytes.read_f64::<BigEndian>()?;
    let y = bytes.read_f64::<BigEndian>()?;

    let order_type = OrderType::AttackMove(MoveGroup::new((x as f64, y as f64)));
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, bytes)
}

fn read_build_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let unit_type_id_num = bytes.read_u16::<BigEndian>()?;
    let x = bytes.read_f64::<BigEndian>()?;
    let y = bytes.read_f64::<BigEndian>()?;

    let unit_type_id = unsafe { UnitTypeID::usize_wrap(unit_type_id_num as usize) };

    let order_type = OrderType::Build(BuildGroup::new(unit_type_id, BuildTarget::Point((x, y))));
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, bytes)
}

fn read_attack_target_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let target_id_num = bytes.read_u16::<BigEndian>()?;

    let target_id = unsafe { UnitID::usize_wrap(target_id_num as usize) };

    let (x, y) = game.units.xy(target_id);
    let unit_target = UnitTarget::new(&game.units, target_id);
    let order_type = OrderType::AttackTarget(MoveGroup::new((x as f64, y as f64)), unit_target);
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, bytes)
}
