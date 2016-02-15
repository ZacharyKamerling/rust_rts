use movement::{Angle,normalize};
use data::aliases::*;

pub struct Missile;

pub struct Missiles {
    available_ids:                  UIDPool<MissileID>,
    pub target:                     VecUID<MissileID,Target>,
    pub facing:                     VecUID<MissileID,Angle>,
    pub turn_rate:                  VecUID<MissileID,Angle>,
    pub x:                          VecUID<MissileID,f32>,
    pub y:                          VecUID<MissileID,f32>,
    pub speed:                      VecUID<MissileID,f32>,
    pub max_travel_dist:            VecUID<MissileID,f32>,
    pub damage:                     VecUID<MissileID,f32>,
    pub damage_radius:              VecUID<MissileID,f32>,
}

impl Missiles {
    pub fn new(num: usize) -> Missiles {
        Missiles {
            available_ids:      UIDPool::new(num),
            target:             VecUID::full_vec(num, Target::NoTarget),
            facing:             VecUID::full_vec(num, normalize(0.0)),
            turn_rate:          VecUID::full_vec(num, normalize(0.0)),
            x:                  VecUID::full_vec(num, 0.0),
            y:                  VecUID::full_vec(num, 0.0),
            speed:              VecUID::full_vec(num, 0.0),
            max_travel_dist:    VecUID::full_vec(num, 0.0),
            damage:             VecUID::full_vec(num, 0.0),
            damage_radius:      VecUID::full_vec(num, 0.0),
        }
    }

    pub fn make_missile(&mut self) -> Option<MissileID> {
        self.available_ids.get_id()
    }

    pub fn kill_missile(&mut self, id: MissileID) {
        self.available_ids.put_id(id);
    }

    pub fn iter(&self) -> Vec<MissileID>
    {
        self.available_ids.iter()
    }
}