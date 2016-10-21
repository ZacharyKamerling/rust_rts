use libs::movement::{Angle,normalize};
use data::aliases::*;

pub struct Missile {
    pub name:                   &'static str,
    pub speed:                  f32,
    pub max_travel_dist:        f32,
    pub turn_rate:              f32,
    pub damage:                 Damage,
    pub damage_type:            DamageType,
}

pub struct Missiles {
    available_ids:                  UIDPool<MissileID>,
    prototypes:                     Vec<Missile>,
    pub missile_type:               VecUID<MissileID,MissileTypeID>,
    pub target:                     VecUID<MissileID,Target>,
    pub facing:                     VecUID<MissileID,Angle>,
    pub turn_rate:                  VecUID<MissileID,f32>,
    pub xy:                         VecUID<MissileID,(f32,f32)>,
    pub speed:                      VecUID<MissileID,f32>,
    pub travel_dist:                VecUID<MissileID,f32>,
    pub max_travel_dist:            VecUID<MissileID,f32>,
    pub damage:                     VecUID<MissileID,Damage>,
    pub damage_type:                VecUID<MissileID,DamageType>,
    pub team:                       VecUID<MissileID,TeamID>,
    pub target_type:                VecUID<MissileID,TargetType>,
}

impl Missiles {
    pub fn new(num: usize, prototypes: Vec<Missile>) -> Missiles {
        Missiles {
            available_ids:      UIDPool::new(num),
            prototypes:         prototypes,
            missile_type:       VecUID::full_vec(num, 0),
            target:             VecUID::full_vec(num, Target::None),
            facing:             VecUID::full_vec(num, normalize(0.0)),
            turn_rate:          VecUID::full_vec(num, 0.0),
            xy:                 VecUID::full_vec(num, (0.0,0.0)),
            speed:              VecUID::full_vec(num, 0.0),
            travel_dist:        VecUID::full_vec(num, 0.0),
            max_travel_dist:    VecUID::full_vec(num, 0.0),
            damage:             VecUID::full_vec(num, Damage::Single(0.0)),
            damage_type:        VecUID::full_vec(num, DamageType::SmallBlast),
            team:               VecUID::full_vec(num, unsafe { TeamID::usize_wrap(0) }),
            target_type:        VecUID::full_vec(num, TargetType::Ground),
        }
    }

    pub fn make_missile(&mut self, missile_type: MissileTypeID) -> Option<MissileID> {
        let fps = FPS as f32;
        match self.available_ids.get_id() {
            Some(id) => {
                let proto = &self.prototypes[missile_type];

                self.missile_type[id]       = missile_type;
                self.speed[id]              = proto.speed / fps;
                self.damage[id]             = proto.damage;
                self.damage_type[id]        = proto.damage_type;
                self.turn_rate[id]          = proto.turn_rate / fps;
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