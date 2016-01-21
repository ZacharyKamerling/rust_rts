extern crate rand;

use std::rc::Rc;
use movement::{Angle,normalize};
use std::collections::{HashSet};
use std::collections::vec_deque::{VecDeque};

use data::aliases::*;
use data::game::{Game};
use data::kdt_point::{KDTPoint};
use data::weapons::{Weapon,make_weapon};
use useful_bits::{full_vec};

pub struct Unit {
    pub unit_type:                  usize,
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
    pub weapons:                    Vec<Weapon>,
    pub is_flying:                  bool,
    pub is_structure:               bool,
    pub is_ground:                  bool,
    pub is_automatic:               bool,
}

pub struct Units {
    pub available_ids:              VecDeque<UnitID>,
    // IDENTITY
    pub unit_type:                  Vec<UnitTypeID>,
    pub team:                       Vec<TeamID>,
    pub anim:                       Vec<AnimID>,
    pub alive:                      Vec<bool>,
    pub encoding:                   Vec<Vec<u8>>,
    // MOVEMENT
    pub x:                          Vec<f32>,
    pub y:                          Vec<f32>,
    pub radius:                     Vec<f32>,
    pub weight:                     Vec<f32>,
    pub speed:                      Vec<f32>,
    pub top_speed:                  Vec<f32>,
    pub acceleration:               Vec<f32>,
    pub deceleration:               Vec<f32>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub path:                       Vec<Vec<(isize,isize)>>,
    pub width_and_height:           Vec<Option<(isize,isize)>>,
    // STATS
    pub health:                     Vec<f32>,
    pub health_regen:               Vec<f32>,
    pub max_health:                 Vec<f32>,
    pub progress:                   Vec<f32>,
    pub progress_required:          Vec<f32>,
    // PRODUCTION
    pub build_rate:                 Vec<f32>,
    pub build_range:                Vec<f32>,
    pub build_roster:               Vec<Rc<HashSet<UnitTypeID>>>,
    // COMBAT ORIENTED
    pub weapons:                    Vec<Vec<WeaponID>>,
    pub orders:                     Vec<VecDeque<Order>>,
    pub passengers:                 Vec<Vec<UnitID>>,
    pub capacity:                   Vec<usize>,
    pub size:                       Vec<usize>,
    pub sight_range:                Vec<f32>,
    // FLAGS
    pub is_flying:                  Vec<bool>,
    pub is_structure:               Vec<bool>,
    pub is_ground:                  Vec<bool>,
    pub is_moving:                  Vec<bool>,
    pub is_automatic:               Vec<bool>,
    // MUTABLE FLAGS
    pub is_stealthed:               Vec<usize>,
    // OTHER
    pub active_range:               Vec<f32>,
    pub in_range:                   Vec<Vec<KDTPoint>>,
}

impl Units {
    pub fn new(num: usize) -> Units {
        let mut available_ids = VecDeque::with_capacity(num);
        let empty_roster = Rc::new(HashSet::new());
        let mut c: usize = num;

        while c > 0 {
            c -= 1;
            available_ids.push_front(c);
        }

        Units {
            available_ids:          available_ids,
            encoding:               full_vec(num, Vec::new()),
            unit_type:              full_vec(num, 0),
            team:                   full_vec(num, 0),
            anim:                   full_vec(num, 0),
            alive:                  full_vec(num, false),
            x:                      full_vec(num, 0.0),
            y:                      full_vec(num, 0.0),
            radius:                 full_vec(num, 0.0),
            weight:                 full_vec(num, 0.0),
            speed:                  full_vec(num, 0.0),
            top_speed:              full_vec(num, 0.0),
            acceleration:           full_vec(num, 0.0),
            deceleration:           full_vec(num, 0.0),
            facing:                 full_vec(num, normalize(0.0)),
            turn_rate:              full_vec(num, normalize(0.0)),
            path:                   full_vec(num, Vec::new()),
            health:                 full_vec(num, 0.0),
            health_regen:           full_vec(num, 0.0),
            max_health:             full_vec(num, 0.0),
            progress:               full_vec(num, 0.0),
            progress_required:      full_vec(num, 0.0),
            orders:                 full_vec(num, VecDeque::new()),
            build_rate:             full_vec(num, 0.0),
            build_range:            full_vec(num, 0.0),
            build_roster:           full_vec(num, empty_roster.clone()),
            weapons:                full_vec(num, Vec::new()),
            passengers:             full_vec(num, Vec::new()),
            capacity:               full_vec(num, 0),
            size:                   full_vec(num, 0),
            is_ground:              full_vec(num, true),
            is_flying:              full_vec(num, false),
            is_structure:           full_vec(num, false),
            is_moving:              full_vec(num, false),
            is_automatic:           full_vec(num, false),
            is_stealthed:           full_vec(num, 0),
            active_range:           full_vec(num, 0.0),
            sight_range:            full_vec(num, 0.0),
            width_and_height:       full_vec(num, None),
            in_range:               full_vec(num, Vec::new()),
        }
    }
}

pub fn make_unit(game: &mut Game, proto: &Unit) -> Option<UnitID> {
    match game.units.available_ids.pop_front() {
        Some(id) => {
            // Special Stats
            game.units.encoding[id].clear();
            game.units.path[id].clear();
            game.units.weapons[id].clear();
            game.units.alive[id]                = true;
            game.units.anim[id]                 = 0;
            game.units.progress[id]             = 0.0;
            game.units.speed[id]                = 0.0;
            game.units.health[id]               = proto.max_health;
            // Proto Stats
            game.units.unit_type[id]            = proto.unit_type;
            game.units.radius[id]               = proto.radius;
            game.units.weight[id]               = proto.weight;
            game.units.top_speed[id]            = proto.top_speed;
            game.units.acceleration[id]         = proto.acceleration;
            game.units.deceleration[id]         = proto.deceleration;
            game.units.turn_rate[id]            = proto.turn_rate;
            game.units.health_regen[id]         = proto.health_regen;
            game.units.max_health[id]           = proto.max_health;
            game.units.progress_required[id]    = proto.progress_required;
            game.units.build_rate[id]           = proto.build_rate;
            game.units.build_range[id]          = proto.build_range;
            game.units.build_roster[id]         = proto.build_roster.clone();
            game.units.sight_range[id]          = proto.sight_range;
            game.units.is_flying[id]            = proto.is_flying;
            game.units.is_structure[id]         = proto.is_structure;
            game.units.is_ground[id]            = proto.is_ground;
            game.units.is_automatic[id]         = proto.is_automatic;

            for wpn_proto in proto.weapons.iter() {
                let wpn_id = make_weapon(game, wpn_proto, id);
                game.units.weapons[id].push(wpn_id);
            }

            Some(id)
        }
        None => None
    }
}