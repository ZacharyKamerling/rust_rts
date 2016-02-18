extern crate rand;

use std::rc::Rc;
use movement::{Angle,normalize};
use std::collections::{HashSet};
use std::collections::vec_deque::{VecDeque};
use data::aliases::*;
use data::kdt_point::{KDTUnit};
use data::weapons::{Weapon,Weapons};
use data::move_groups::{MoveGroups};

pub struct Unit {
    pub name:                       &'static str,
    pub radius:                     f32,
    pub weight:                     f32,
    pub top_speed:                  f32,
    pub acceleration:               f32,
    pub deceleration:               f32,
    pub turn_rate:                  f32,
    pub health_regen:               f32,
    pub max_health:                 f32,
    pub progress_required:          f32,
    pub build_rate:                 f32,
    pub build_range:                f32,
    pub build_roster:               Rc<HashSet<UnitTypeID>>,
    pub sight_range:                f32,
    pub radar_range:                f32,
    pub active_range:               f32,
    pub weapons:                    Vec<Weapon>,
    pub target_type:                TargetType,
    pub is_automatic:               bool,
}

pub struct Units {
    available_ids:                  UIDPool<UnitID>,
    prototypes:                     Vec<Unit>,
    pub move_groups:                MoveGroups,
    // IDENTITY
    pub unit_type:                  VecUID<UnitID,UnitTypeID>,
    pub team:                       VecUID<UnitID,TeamID>,
    pub anim:                       VecUID<UnitID,AnimID>,
    pub encoding:                   VecUID<UnitID,Vec<u8>>,
    // MOVEMENT
    pub x:                          VecUID<UnitID,f32>,
    pub y:                          VecUID<UnitID,f32>,
    pub x_repulsion:                VecUID<UnitID,f32>,
    pub y_repulsion:                VecUID<UnitID,f32>,
    pub radius:                     VecUID<UnitID,f32>,
    pub weight:                     VecUID<UnitID,f32>,
    pub speed:                      VecUID<UnitID,f32>,
    pub top_speed:                  VecUID<UnitID,f32>,
    pub acceleration:               VecUID<UnitID,f32>,
    pub deceleration:               VecUID<UnitID,f32>,
    pub facing:                     VecUID<UnitID,Angle>,
    pub turn_rate:                  VecUID<UnitID,Angle>,
    pub path:                       VecUID<UnitID,Vec<(isize,isize)>>,
    pub width_and_height:           VecUID<UnitID,Option<(isize,isize)>>,
    // STATS
    pub health:                     VecUID<UnitID,f32>,
    pub health_regen:               VecUID<UnitID,f32>,
    pub max_health:                 VecUID<UnitID,f32>,
    pub progress:                   VecUID<UnitID,f32>,
    pub progress_required:          VecUID<UnitID,f32>,
    // PRODUCTION
    pub build_rate:                 VecUID<UnitID,f32>,
    pub build_range:                VecUID<UnitID,f32>,
    pub build_roster:               VecUID<UnitID,Rc<HashSet<UnitTypeID>>>,
    // COMBAT ORIENTED
    pub weapons:                    VecUID<UnitID,Vec<WeaponID>>,
    pub orders:                     VecUID<UnitID,VecDeque<Order>>,
    pub passengers:                 VecUID<UnitID,Vec<UnitID>>,
    pub capacity:                   VecUID<UnitID,usize>,
    pub size:                       VecUID<UnitID,usize>,
    pub sight_range:                VecUID<UnitID,f32>,
    pub radar_range:                VecUID<UnitID,f32>,
    // FLAGS
    pub target_type:                VecUID<UnitID,TargetType>,
    pub is_automatic:               VecUID<UnitID,bool>,
    // MUTABLE FLAGS
    pub is_stealthed:               VecUID<UnitID,usize>,
    // OTHER
    pub active_range:               VecUID<UnitID,f32>,
    pub in_range:                   VecUID<UnitID,Vec<KDTUnit>>,
}

impl Units {
    pub fn new(num: usize, prototypes: Vec<Unit>) -> Units {
        let available_ids = UIDPool::new(num);
        let empty_roster = Rc::new(HashSet::new());

        Units {
            available_ids:          available_ids,
            prototypes:             prototypes,
            move_groups:            MoveGroups::new(),
            encoding:               VecUID::full_vec(num, Vec::new()),
            unit_type:              VecUID::full_vec(num, unsafe { UnitTypeID::usize_wrap(0) }),
            team:                   VecUID::full_vec(num, unsafe { TeamID::usize_wrap(0) }),
            anim:                   VecUID::full_vec(num, 0),
            x:                      VecUID::full_vec(num, 0.0),
            y:                      VecUID::full_vec(num, 0.0),
            x_repulsion:            VecUID::full_vec(num, 0.0),
            y_repulsion:            VecUID::full_vec(num, 0.0),
            radius:                 VecUID::full_vec(num, 0.0),
            weight:                 VecUID::full_vec(num, 0.0),
            speed:                  VecUID::full_vec(num, 0.0),
            top_speed:              VecUID::full_vec(num, 0.0),
            acceleration:           VecUID::full_vec(num, 0.0),
            deceleration:           VecUID::full_vec(num, 0.0),
            facing:                 VecUID::full_vec(num, normalize(0.0)),
            turn_rate:              VecUID::full_vec(num, normalize(0.0)),
            path:                   VecUID::full_vec(num, Vec::new()),
            health:                 VecUID::full_vec(num, 0.0),
            health_regen:           VecUID::full_vec(num, 0.0),
            max_health:             VecUID::full_vec(num, 0.0),
            progress:               VecUID::full_vec(num, 0.0),
            progress_required:      VecUID::full_vec(num, 0.0),
            orders:                 VecUID::full_vec(num, VecDeque::new()),
            build_rate:             VecUID::full_vec(num, 0.0),
            build_range:            VecUID::full_vec(num, 0.0),
            build_roster:           VecUID::full_vec(num, empty_roster.clone()),
            weapons:                VecUID::full_vec(num, Vec::new()),
            passengers:             VecUID::full_vec(num, Vec::new()),
            capacity:               VecUID::full_vec(num, 0),
            size:                   VecUID::full_vec(num, 0),
            target_type:            VecUID::full_vec(num, TargetType::Ground),
            is_automatic:           VecUID::full_vec(num, false),
            is_stealthed:           VecUID::full_vec(num, 0),
            active_range:           VecUID::full_vec(num, 0.0),
            sight_range:            VecUID::full_vec(num, 0.0),
            radar_range:            VecUID::full_vec(num, 0.0),
            width_and_height:       VecUID::full_vec(num, None),
            in_range:               VecUID::full_vec(num, Vec::new()),
        }
    }

    pub fn kill_unit(&mut self, id: UnitID) {
        self.available_ids.put_id(id);
        self.is_automatic[id] = true;
    }

    pub fn make_unit(&mut self, fps: f32, wpns: &mut Weapons, unit_type: UnitTypeID) -> Option<UnitID> {
        let proto = &self.prototypes[unsafe { unit_type.usize_unwrap() }];
        match self.available_ids.get_id() {
            Some(id) => {
                // Special Stats
                self.encoding[id].clear();
                self.path[id].clear();
                self.weapons[id].clear();
                self.unit_type[id]            = unit_type;
                self.anim[id]                 = 0;
                self.progress[id]             = 0.0;
                self.speed[id]                = 0.0;
                self.x_repulsion[id]          = 0.0;
                self.y_repulsion[id]          = 0.0;
                self.health[id]               = proto.max_health;
                // Proto Stats
                self.radius[id]               = proto.radius;
                self.weight[id]               = proto.weight;
                self.top_speed[id]            = proto.top_speed / fps;
                self.acceleration[id]         = proto.acceleration / (fps * fps);
                self.deceleration[id]         = proto.deceleration / (fps * fps);
                self.turn_rate[id]            = normalize(proto.turn_rate / fps);
                self.health_regen[id]         = proto.health_regen / fps;
                self.max_health[id]           = proto.max_health;
                self.progress_required[id]    = proto.progress_required;
                self.build_rate[id]           = proto.build_rate / fps;
                self.build_range[id]          = proto.build_range;
                self.build_roster[id]         = proto.build_roster.clone();
                self.active_range[id]         = proto.active_range;
                self.sight_range[id]          = proto.sight_range;
                self.radar_range[id]          = proto.radar_range;
                self.target_type[id]          = proto.target_type;
                self.is_automatic[id]         = proto.is_automatic;

                for wpn_proto in proto.weapons.iter() {
                    let wpn_id = wpns.make_weapon(wpn_proto, id);
                    self.weapons[id].push(wpn_id);
                }

                Some(id)
            }
            None => None
        }
    }

    pub fn iter(&self) -> Vec<UnitID>
    {
        self.available_ids.iter()
    }
}