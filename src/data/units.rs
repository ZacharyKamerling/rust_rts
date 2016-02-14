extern crate rand;

use std::rc::Rc;
use movement::{Angle,normalize};
use std::collections::{HashSet};
use std::collections::vec_deque::{VecDeque};

use data::aliases::*;
use data::kdt_point::{KDTPoint};
use data::weapons::{Weapon,Weapons};
use data::move_groups::{MoveGroups};
use useful_bits::{full_vec};

pub struct Unit {
    pub unit_type:                  UnitTypeID,
    pub radius:                     f32,
    pub weight:                     f32,
    pub top_speed:                  f32,
    pub acceleration:               f32,
    pub deceleration:               f32,
    pub turn_rate:                  Angle,
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
    pub is_flying:                  bool,
    pub is_structure:               bool,
    pub is_ground:                  bool,
    pub is_automatic:               bool,
}

pub struct Units {
    available_ids:                  VecDeque<UnitID>,
    pub move_groups:                MoveGroups,
    // IDENTITY
    pub unit_type:                  VecUID<UnitID,UnitTypeID>,
    pub team:                       VecUID<UnitID,TeamID>,
    pub anim:                       VecUID<UnitID,AnimID>,
    pub alive:                      VecUID<UnitID,bool>,
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
    pub is_flying:                  VecUID<UnitID,bool>,
    pub is_structure:               VecUID<UnitID,bool>,
    pub is_ground:                  VecUID<UnitID,bool>,
    pub is_automatic:               VecUID<UnitID,bool>,
    // MUTABLE FLAGS
    pub is_stealthed:               VecUID<UnitID,usize>,
    // OTHER
    pub active_range:               VecUID<UnitID,f32>,
    pub in_range:                   VecUID<UnitID,Vec<KDTPoint>>,
}

impl Units {
    pub fn new(num: usize) -> Units {
        let mut available_ids = VecDeque::with_capacity(num);
        let empty_roster = Rc::new(HashSet::new());
        let mut c: usize = num;

        while c > 0 {
            c -= 1;
            available_ids.push_front(UnitID::unsafe_wrap(c));
        }

        Units {
            available_ids:          available_ids,
            move_groups:            MoveGroups::new(),
            encoding:               VecUID::full_vec(num, Vec::new()),
            unit_type:              VecUID::full_vec(num, 0),
            team:                   VecUID::full_vec(num, TeamID::unsafe_wrap(0)),
            anim:                   VecUID::full_vec(num, 0),
            alive:                  VecUID::full_vec(num, false),
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
            is_ground:              VecUID::full_vec(num, true),
            is_flying:              VecUID::full_vec(num, false),
            is_structure:           VecUID::full_vec(num, false),
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
        self.available_ids.push_back(id);
        self.alive[id] = false;
    }

    pub fn make_unit(&mut self, wpns: &mut Weapons, proto: &Unit) -> Option<UnitID> {
        match self.available_ids.pop_front() {
            Some(id) => {
                // Special Stats
                self.encoding[id].clear();
                self.path[id].clear();
                self.weapons[id].clear();
                self.alive[id]                = true;
                self.anim[id]                 = 0;
                self.progress[id]             = 0.0;
                self.speed[id]                = 0.0;
                self.x_repulsion[id]          = 0.0;
                self.y_repulsion[id]          = 0.0;
                self.health[id]               = proto.max_health;
                // Proto Stats
                self.unit_type[id]            = proto.unit_type;
                self.radius[id]               = proto.radius;
                self.weight[id]               = proto.weight;
                self.top_speed[id]            = proto.top_speed;
                self.acceleration[id]         = proto.acceleration;
                self.deceleration[id]         = proto.deceleration;
                self.turn_rate[id]            = proto.turn_rate;
                self.health_regen[id]         = proto.health_regen;
                self.max_health[id]           = proto.max_health;
                self.progress_required[id]    = proto.progress_required;
                self.build_rate[id]           = proto.build_rate;
                self.build_range[id]          = proto.build_range;
                self.build_roster[id]         = proto.build_roster.clone();
                self.active_range[id]         = proto.active_range;
                self.sight_range[id]          = proto.sight_range;
                self.radar_range[id]          = proto.radar_range;
                self.is_flying[id]            = proto.is_flying;
                self.is_structure[id]         = proto.is_structure;
                self.is_ground[id]            = proto.is_ground;
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
        let alive = |id: usize| {
            if self.alive[UnitID::unsafe_wrap(id)] {
                Some(UnitID::unsafe_wrap(id))
            }
            else {
                None
            }
        };
        (0..self.alive.len()).filter_map(&alive).collect()
    }
}