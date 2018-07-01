extern crate rand;
extern crate byteorder;
extern crate num;

use libs::kdt::KDTree;
use libs::bytegrid::ByteGrid;
use libs::netcom::{Netcom, send_message_to_player};
use libs::tmx_decode::MapData;
use self::rand::ThreadRng;
use self::rand::Rng;
use self::byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use self::num::FromPrimitive;
use std::sync::{Arc, Mutex};
use std::io::Cursor;
use std::io;
use data::logger::Logger;
use data::units::{Units, Unit, Missiles, Missile};
use data::kdt_point::{KDTUnit, KDTMissile};
use data::teams::Teams;
use data::move_groups::MoveGroup;
use data::build_groups::{BuildGroup, BuildTarget};
use std::collections::{HashSet};
use std::iter::FromIterator;
use std::rc::Rc;
use data::aliases::*;

#[derive(Clone)]
pub struct Game {
    fps: f64,
    max_units: usize,
    max_weapons: usize,
    max_missiles: usize,
    encoded_map_data: Vec<u8>,
    encoded_unit_info: Vec<u8>,
    encoded_missile_info: Vec<u8>,
    pub rng: ThreadRng,
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
        unit_id_map: UIDMapping<UnitTypeID>,
        missile_prototypes: VecUID<MissileTypeID, Missile>,
        missile_id_map: UIDMapping<MissileTypeID>,
        encoded_unit_info: Vec<u8>,
        encoded_missile_info: Vec<u8>,
        netcom: Arc<Mutex<Netcom>>,
    ) -> Game {
        let (width, height) = map_data.width_and_height();

        Game {
            fps: 10.0,
            max_units: max_units,
            max_weapons: max_units * 2,
            max_missiles: max_units * 4,
            rng: rand::thread_rng(),
            encoded_map_data: map_data.encode(),
            encoded_unit_info: encoded_unit_info,
            encoded_missile_info: encoded_missile_info,
            map_data: map_data,
            units: Units::new(max_units, unit_prototypes, unit_id_map),
            missiles: Missiles::new(max_units * 4, missile_prototypes, missile_id_map),
            teams: Teams::new(max_units, max_teams, width, height),
            unit_kdt: KDTree::new(Vec::new()),
            missile_kdt: KDTree::new(Vec::new()),
            bytegrid: ByteGrid::new(width as isize, height as isize),
            logger: Logger::new(),
            netcom: netcom,
            frame_number: 0,
        }
    }

    pub fn fps(&self) -> f64 {
        self.fps
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
    pub fn get_random_collision_offset(&mut self) -> f64 {
        self.rng.gen_range(-0.000001, 0.000001)
    }
}

pub fn incorporate_messages(game: &mut Game, msgs: Vec<(String, usize, Vec<u8>)>) {
    for msg in msgs {
        let (name, team, data) = msg;
        let bytes = &mut Cursor::new(data);

        if let Ok(Some(msg_type)) = bytes.read_u8().map(ServerMessage::from_u8) {
            let opt_order_id = unsafe {
                bytes.read_u32::<BigEndian>().map(|a| {
                    OrderID::usize_wrap(a as usize)
                })
            };

            if let Ok(order_id) = opt_order_id {
                let team_id = unsafe { TeamID::usize_wrap(team) };

                match msg_type {
                    ServerMessage::Move => {
                        let _ = read_move_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::AttackTarget => {
                        let _ = read_attack_target_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::Build => {
                        let _ = read_build_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::Train => {
                        let _ = read_train_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::Assist => {
                        let _ = read_assist_message(game, order_id, team_id, bytes);
                    }
					ServerMessage::Stop => {
                        let _ = read_stop_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::AttackMove => {
                        let _ = read_attack_move_message(game, order_id, team_id, bytes);
                    }
                    ServerMessage::MapInfoRequest => {
                        send_tilegrid_info(game, team_id, name);
                    }
                    ServerMessage::UnitInfoRequest => {
                        send_unit_info(game, name);
                    }
                    ServerMessage::MissileInfoRequest => {
                        send_missile_info(game, name);
                    }
                }
            }
        }
    }
}

fn get_order_units(game: &Game, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<Vec<UnitID>> {
    let mut vec = Vec::new();

    while let Ok(uid) = bytes.read_u16::<BigEndian>() {
        let unit_id = unsafe { UnitID::usize_wrap(uid as usize) };

        if (uid as usize) < game.max_units &&
            game.units.team(unit_id) == team_id &&
            !game.units.is_automatic(unit_id)
        {
            vec.push(unit_id);
        }
    }

    Ok(vec)
}

fn add_order_to_units(game: &mut Game, team_id: TeamID, order: Rc<Order>, units: Vec<UnitID>, queue_order: QueueOrder) {
    for unit_id in units {
        let uid = unsafe {
            unit_id.usize_unwrap() as usize
        };

        if uid < game.max_units && game.units.team(unit_id) == team_id && !game.units.is_automatic(unit_id)
        {
            match queue_order {
                QueueOrder::Replace => {
                    game.units.mut_orders(unit_id).clear();
                    game.units.mut_orders(unit_id).push_back(order.clone());
                }
                QueueOrder::Append => {
                    game.units.mut_orders(unit_id).push_back(order.clone());
                }
                QueueOrder::Prepend => {
                    game.units.mut_orders(unit_id).push_front(order.clone());
                }
                QueueOrder::Clear => {
                    game.units.mut_orders(unit_id).clear();
                }
            }
        }
    }
}

fn send_unit_info(game: &Game, name: String) {
    // We add 5 bytes to the encoded data for the frame number and message tag
    let len = game.encoded_unit_info.len() + 5;
    let mut msg = Cursor::new(Vec::with_capacity(len));
    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let mut bytes = msg.into_inner();
    bytes.append(&mut game.encoded_unit_info.clone());

    send_message_to_player(game.netcom.clone(), bytes, name);
}

fn send_missile_info(game: &Game, name: String) {
    // We add 5 bytes to the encoded data for the frame number and message tag
    let len = game.encoded_missile_info.len() + 5;
    let mut msg = Cursor::new(Vec::with_capacity(len));
    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let mut bytes = msg.into_inner();
    bytes.append(&mut game.encoded_missile_info.clone());

    send_message_to_player(game.netcom.clone(), bytes, name);
}

fn send_tilegrid_info(game: &Game, team: TeamID, name: String) {
    // We add 6 bytes to the encoded data for the frame number, tag, & team
    let team_usize = unsafe { team.usize_unwrap() };
    let len = game.encoded_map_data.len() + 6;
    let mut msg = Cursor::new(Vec::with_capacity(len));
    let _ = msg.write_u32::<BigEndian>(game.frame_number);
    let _ = msg.write_u8(ClientMessage::MapInfo as u8);
    let _ = msg.write_u8(team_usize as u8);
    let mut bytes = msg.into_inner();
    bytes.append(&mut game.encoded_map_data.clone());

    send_message_to_player(game.netcom.clone(), bytes, name);
}

fn read_move_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let x = bytes.read_f64::<BigEndian>()?;
    let y = bytes.read_f64::<BigEndian>()?;
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    let membership = HashSet::from_iter(units.iter().cloned().map(|id| game.units.new_unit_target(id)));
    let order_type = OrderType::Move(MoveGroup::new((x as f64, y as f64), membership));
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
}

fn read_attack_move_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let x = bytes.read_f64::<BigEndian>()?;
    let y = bytes.read_f64::<BigEndian>()?;
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    let membership = HashSet::from_iter(units.iter().cloned().map(|id| game.units.new_unit_target(id)));
    let order_type = OrderType::AttackMove(MoveGroup::new((x as f64, y as f64), membership));
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
}

fn read_attack_target_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let target_id_num = bytes.read_u16::<BigEndian>()?;
    let target_id = unsafe { UnitID::usize_wrap(target_id_num as usize) };
    let (x, y) = game.units.xy(target_id);
    let unit_target = game.units.new_unit_target(target_id);
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    let membership = HashSet::from_iter(units.iter().cloned().map(|id| game.units.new_unit_target(id)));
    let order_type = OrderType::AttackTarget(MoveGroup::new((x as f64, y as f64), membership), unit_target);
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
}

fn read_assist_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let target_id_num = bytes.read_u16::<BigEndian>()?;
    let target_id = unsafe { UnitID::usize_wrap(target_id_num as usize) };
    let unit_target = game.units.new_unit_target(target_id);
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    let order_type = OrderType::Assist(unit_target);
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
}

fn read_stop_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    let order_type = OrderType::Stop;
    let order = Rc::new(Order {
        order_type: order_type,
        order_id: order_id,
    });

    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
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
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let units = get_order_units(game, team_id, bytes)?;
    add_order_to_units(game, team_id, order, units, queue_order);

    Ok(())
}

fn read_train_message(game: &mut Game, order_id: OrderID, team_id: TeamID, bytes: &mut Cursor<Vec<u8>>) -> io::Result<()> {
    let unit_type_id_num = bytes.read_u16::<BigEndian>()?;
    let unit_type_id = unsafe { UnitTypeID::usize_wrap(unit_type_id_num as usize) };
    let repeat: bool = bytes.read_u8()? == 1;
    let queue_order = QueueOrder::from_u8(bytes.read_u8()?).unwrap();
    let trainers = get_order_units(game, team_id, bytes)?;
    let train_order = TrainOrder {
        order_id: order_id,
        unit_type: unit_type_id,
        repeat: repeat,
    };

    add_training_to_units(game, team_id, train_order, trainers, queue_order);

    Ok(())
}

fn add_training_to_units(game: &mut Game, team_id: TeamID, train_order: TrainOrder, units: Vec<UnitID>, queue_order: QueueOrder) {
    for unit_id in units {
        let uid = unsafe {
            unit_id.usize_unwrap() as usize
        };

        if uid < game.max_units && game.units.team(unit_id) == team_id && !game.units.is_automatic(unit_id) && game.units.is_structure(unit_id)
        {
            match queue_order {
                QueueOrder::Replace => {
                    refund_training_prime(game, unit_id, team_id);
                    game.units.mut_train_queue(unit_id).clear();
                    game.units.mut_train_queue(unit_id).push_back(train_order);
                }
                QueueOrder::Append => {
                    game.units.mut_train_queue(unit_id).push_back(train_order);
                }
                QueueOrder::Prepend => {
                    refund_training_prime(game, unit_id, team_id);
                    game.units.mut_train_queue(unit_id).push_front(train_order);
                }
                QueueOrder::Clear => {
                    refund_training_prime(game, unit_id, team_id);
                    game.units.mut_train_queue(unit_id).clear();
                }
            }
        }
    }
}

fn refund_training_prime(game: &mut Game, trainer: UnitID, team: TeamID) {
    let train_order_front = game.units.train_queue(trainer).front().cloned();
    if let Some(train_order) = train_order_front {
        let proto = game.units.proto(train_order.unit_type);
        let build_cost = proto.build_cost();
        let prime_cost = proto.prime_cost();
        let progress = game.units.train_progress(trainer);
        let refund_ratio = progress / build_cost;

        game.teams.prime[team] += prime_cost * refund_ratio;
    }
}