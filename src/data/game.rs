extern crate rand;

use kdt::{KDTree};
use bytegrid::{ByteGrid};
use self::rand::distributions::{Sample,Range};
use self::rand::ThreadRng;

use data::aliases::*;
use data::units::{Units,Unit,make_unit};
use data::kdt_point::{KDTPoint};
use data::teams::{Teams};
use data::weapons::{Weapons,Weapon};
use data::missiles::{Missiles,Missile};
use data::event_handlers::{EventHandlers};

pub struct Game {
    pub game_rng:                   ThreadRng,
    pub random_offset_gen:          Range<f32>,
    pub event_handlers:             EventHandlers,
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
    pub fn new(width: usize, height: usize) -> Game {
        let num = 2048;
        Game {
            game_rng: rand::thread_rng(),
            random_offset_gen: Range::new(-0.00001, 0.00001),
            event_handlers: EventHandlers::new(num),
            unit_blueprints: Vec::new(),
            weapon_blueprints: Vec::new(),
            missile_blueprints: Vec::new(),
            units: Units::new(num),
            weapons: Weapons::new(num * 2),
            missiles: Missiles::new(num * 8),
            teams: Teams::new(16, width, height),
            kdt: KDTree::new(Vec::new()),
            bytegrid: ByteGrid::new(width as isize, height as isize),
        }
    }

    fn kill_unit(&mut self, id: UnitID) -> () {
        self.units.available_ids.push(id);
        self.units.alive[id] = false;
    }

    pub fn spawn_unit(&mut self, proto: &Unit, parent: UnitID) -> UnitID {
        let id = make_unit(self, proto);
        let x_offset = self.random_offset_gen.sample(&mut self.game_rng);
        let y_offset = self.random_offset_gen.sample(&mut self.game_rng);
        let par_x = self.units.x[parent];
        let par_y = self.units.y[parent];
        let new_x = par_x + x_offset;
        let new_y = par_y + y_offset;
        let (cx,cy) = self.bytegrid.correct_move((par_x, par_y), (new_x, new_y));

        self.units.x[id] = cx;
        self.units.y[id] = cy;
        self.units.facing[id] = self.units.facing[parent];
        id
    }
}