extern crate rand;
extern crate byteorder;

use kdt::{KDTree};
use bytegrid::{ByteGrid};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::io::Cursor;

use data::aliases::*;
use data::units::{Units,Unit,UnitID};
use data::kdt_point::{KDTPoint};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};

pub struct Game {
    fps:                            f32,
    rng:                            ThreadRng,
    random_offset_gen:              Range<f32>,
    pub unit_blueprints:            Vec<Unit>,
    pub weapon_blueprints:          Vec<Weapon>,
    pub missile_blueprints:         Vec<Missile>,
    pub units:                      Units,
    pub weapons:                    Weapons,
    pub missiles:                   Missiles,
    pub teams:                      Teams,
    pub kdt:                        KDTree<KDTPoint>,
    pub bytegrid:                   ByteGrid,
}

impl Game {
    pub fn new(fps: usize, num: usize, width: usize, height: usize) -> Game {
        Game {
            fps: fps as f32,
            rng: rand::thread_rng(),
            random_offset_gen: Range::new(-0.0001, 0.0001),
            unit_blueprints: Vec::new(),
            weapon_blueprints: Vec::new(),
            missile_blueprints: Vec::new(),
            units: Units::new(num),
            weapons: Weapons::new(num * 2),
            missiles: Missiles::new(num * 4),
            teams: Teams::new(num, width, height),
            kdt: KDTree::new(Vec::new()),
            bytegrid: ByteGrid::new(width as isize, height as isize),
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    // Produces a tiny random offset.
    // This is useful to avoid units occupying the same spot and being unable to collide correctly.
    pub fn get_random_offset(&mut self) -> f32 {
        self.random_offset_gen.sample(&mut self.rng)
    }

    pub fn spawn_unit(&mut self, proto: &Unit, parent: UnitID) -> Option<UnitID> {
        let opt_id = self.units.make_unit(&mut self.weapons, proto);

        match opt_id {
            Some(id) => {
                let x_offset = self.get_random_offset();
                let y_offset = self.get_random_offset();
                let par_x = self.units.x[parent];
                let par_y = self.units.y[parent];
                let new_x = par_x + x_offset;
                let new_y = par_y + y_offset;
                let (cx,cy,_,_) = self.bytegrid.correct_move((par_x, par_y), (new_x, new_y));

                self.units.x[id] = cx;
                self.units.y[id] = cy;
                self.units.facing[id] = self.units.facing[parent];
                Some(id)
            }
            None => None
        }
    }

    pub fn incorporate_messages(&mut self, msgs: Vec<(String, usize, Vec<u8>)>) {
        for msg in msgs {
            let (name,team,data) = msg;
            let mut cursor = Cursor::new(data);
            let msg_type = cursor.read_u8();

            match msg_type {
                Ok(0) => { // MOVE COMMAND
                    self.read_move_message(team, &mut cursor);
                }
                _ => {
                    println!("Received poorly formatted message from {}.", name);
                }
            }
        }
    }

    fn read_move_message(&mut self, team: usize, vec: &mut Cursor<Vec<u8>>) {
        let res_ord = vec.read_u8();
        let res_len = vec.read_u16::<BigEndian>();
        let res_x = vec.read_f64::<BigEndian>();
        let res_y = vec.read_f64::<BigEndian>();

        match (res_ord, res_x, res_y, res_len) {
            (Ok(ord), Ok(x), Ok(y), Ok(len)) => {
                let mg_id = self.units.move_groups.make_group(len as usize, x as f32, y as f32);

                for _ in 0..len {
                    let res_uid = vec.read_u16::<BigEndian>();

                    match res_uid {
                        Ok(uid) => {
                            let id = UnitID::wrap(uid as usize);

                            if (uid as usize) < self.units.alive.len() &&
                                self.units.alive[id] &&
                                self.units.team[id] == team &&
                                !self.units.is_automatic[id] &&
                                !self.units.is_structure[id]
                            {
                                match ord {
                                    0 => { // REPLACE
                                        self.clear_units_move_groups(id);
                                        self.units.orders[id].clear();
                                        self.units.orders[id].push_back(Order::Move(mg_id));
                                    }
                                    1 => { // APPEND
                                        self.units.orders[id].push_back(Order::Move(mg_id));
                                    }
                                    2 => { // PREPEND
                                        self.units.orders[id].push_front(Order::Move(mg_id));
                                    }
                                    _ => {
                                        println!("Move message had a wrong order.");
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("Move message wasn't long enough.");
                        }
                    }
                }
            }
            _ => {
                println!("No length in move message.");
            }
        }
    }

    pub fn clear_units_move_groups(&mut self, id: UnitID) {
        for i in 0..self.units.orders[id].len() {
            let ord = self.units.orders[id][i];

            let opt_mg_id = match ord {
                Order::Move(id) => Some(id),
                Order::AttackMove(id) => Some(id),
                Order::AttackTarget(_) => None,
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