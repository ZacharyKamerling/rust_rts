extern crate rand;
extern crate byteorder;

use kdt::{KDTree};
use bytegrid::{ByteGrid};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;
use self::byteorder::{ReadBytesExt, BigEndian};
use std::io::Cursor;

use data::aliases::*;
use data::units::{Units,Unit,make_unit};
use data::kdt_point::{KDTPoint};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};
use data::move_groups::{MoveGroups};

pub struct Game {
    pub game_rng:                   ThreadRng,
    pub random_offset_gen:          Range<f32>,
    pub unit_blueprints:            Vec<Unit>,
    pub weapon_blueprints:          Vec<Weapon>,
    pub missile_blueprints:         Vec<Missile>,
    pub units:                      Units,
    pub weapons:                    Weapons,
    pub missiles:                   Missiles,
    pub teams:                      Teams,
    pub kdt:                        KDTree<KDTPoint>,
    pub bytegrid:                   ByteGrid,
    pub move_groups:                MoveGroups,
}

impl Game {
    pub fn new(num: usize, width: usize, height: usize) -> Game {
        Game {
            game_rng: rand::thread_rng(),
            random_offset_gen: Range::new(-0.0001, 0.0001),
            unit_blueprints: Vec::new(),
            weapon_blueprints: Vec::new(),
            missile_blueprints: Vec::new(),
            units: Units::new(num),
            weapons: Weapons::new(num * 2),
            missiles: Missiles::new(num * 8),
            teams: Teams::new(4, width, height),
            kdt: KDTree::new(Vec::new()),
            bytegrid: ByteGrid::new(width as isize, height as isize),
            move_groups: MoveGroups::new(),
        }
    }

    pub fn kill_unit(&mut self, id: UnitID) -> () {
        self.units.available_ids.push_back(id);
        self.units.alive[id] = false;
    }

    pub fn spawn_unit(&mut self, proto: &Unit, parent: UnitID) -> Option<UnitID> {
        let opt_id = make_unit(self, proto);

        match opt_id {
            Some(id) => {
                let x_offset = self.random_offset_gen.sample(&mut self.game_rng);
                let y_offset = self.random_offset_gen.sample(&mut self.game_rng);
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
                let mg_id = self.move_groups.make_group(len as usize);

                for _ in 0..len {
                    let res_uid = vec.read_u16::<BigEndian>();

                    match res_uid {
                        Ok(uid) => {
                            let id = uid as usize;

                            if  id < self.units.alive.len() &&
                                self.units.alive[id] &&
                                self.units.team[id] == team &&
                                !self.units.is_automatic[id] &&
                                !self.units.is_structure[id]
                            {
                                match ord {
                                    0 => { // REPLACE
                                        self.clear_units_move_groups(id);
                                        self.units.orders[id].clear();
                                        self.units.orders[id].push_back(Order::Move(x as f32, y as f32, mg_id));
                                    }
                                    1 => { // APPEND
                                        self.units.orders[id].push_back(Order::Move(x as f32, y as f32, mg_id));
                                    }
                                    2 => { // PREPEND
                                        self.units.orders[id].push_front(Order::Move(x as f32, y as f32, mg_id));
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

    fn clear_units_move_groups(&mut self, id: usize) {
        for i in 0..self.units.orders[id].len() {
            let ord = self.units.orders[id][i];

            let opt_mg_id = match ord {
                Order::Move(_,_,id) => Some(id),
            };

            match opt_mg_id {
                Some(mg_id) => {
                    self.move_groups.not_in_move_group_anymore(mg_id);
                }
                None => ()
            }
        }
    }
}