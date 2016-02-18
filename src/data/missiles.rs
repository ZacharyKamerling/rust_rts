use movement::{Angle,normalize,denormalize};
use data::aliases::*;

pub struct Missile {
    pub name:                   &'static str,
    pub speed:                  f32,
    pub max_travel_dist:        f32,
    pub damage:                 Damage,
    pub turn_rate:              Angle,
}

pub struct Missiles {
    available_ids:                  UIDPool<MissileID>,
    prototypes:                     Vec<Missile>,
    pub target:                     VecUID<MissileID,Target>,
    pub facing:                     VecUID<MissileID,Angle>,
    pub turn_rate:                  VecUID<MissileID,Angle>,
    pub x:                          VecUID<MissileID,f32>,
    pub y:                          VecUID<MissileID,f32>,
    pub speed:                      VecUID<MissileID,f32>,
    pub travel_dist:                VecUID<MissileID,f32>,
    pub max_travel_dist:            VecUID<MissileID,f32>,
    pub damage:                     VecUID<MissileID,Damage>,
    pub team:                       VecUID<MissileID,TeamID>,
    pub target_type:                VecUID<MissileID,TargetType>,
}

impl Missiles {
    pub fn new(num: usize, prototypes: Vec<Missile>) -> Missiles {
        Missiles {
            available_ids:      UIDPool::new(num),
            prototypes:         prototypes,
            target:             VecUID::full_vec(num, Target::None),
            facing:             VecUID::full_vec(num, normalize(0.0)),
            turn_rate:          VecUID::full_vec(num, normalize(0.0)),
            x:                  VecUID::full_vec(num, 0.0),
            y:                  VecUID::full_vec(num, 0.0),
            speed:              VecUID::full_vec(num, 0.0),
            travel_dist:        VecUID::full_vec(num, 0.0),
            max_travel_dist:    VecUID::full_vec(num, 0.0),
            damage:             VecUID::full_vec(num, Damage::Single(0.0)),
            team:               VecUID::full_vec(num, unsafe { TeamID::usize_wrap(0) }),
            target_type:        VecUID::full_vec(num, TargetType::Ground),
        }
    }

    pub fn make_missile(&mut self, fps: f32, missile_type: MissileTypeID) -> Option<MissileID> {
        match self.available_ids.get_id() {
            Some(id) => {
                let usize_missile_type = unsafe { missile_type.usize_unwrap() };
                let proto = &self.prototypes[usize_missile_type];

                self.speed[id]              = proto.speed / fps;
                self.damage[id]             = proto.damage;
                self.turn_rate[id]          = normalize(denormalize(proto.turn_rate) / fps);
                self.travel_dist[id]        = 0.0;
                self.max_travel_dist[id]    = proto.max_travel_dist;
                self.target_type[id]        = TargetType::Ground;

                Some(id)
            }
            None => None
        }
    }

    pub fn kill_missile(&mut self, id: MissileID) {
        self.available_ids.put_id(id);
    }

    pub fn iter(&self) -> Vec<MissileID>
    {
        self.available_ids.iter()
    }
}