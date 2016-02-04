use std::collections::vec_deque::{VecDeque};
use movement::{Angle,normalize};
use data::aliases::*;
use useful_bits::{full_vec};

pub struct Weapon {
    pub wpn_type:                       usize,
    pub is_bomb_bay:                    bool,
    pub attack_type:                    AttackType,
    pub turn_rate:                      Angle,
    pub lock_offset:                    Angle,
    pub firing_arc:                     Angle,
    pub range:                          f32,
    pub firing_offset:                  f32,
    pub fire_rate:                      f32,
    pub salvo_count:                    usize,
    pub salvo_fire_rate:                f32,
}

pub struct Weapons {
    pub available_ids:              VecDeque<WeaponID>,
    // IDENTITY
    pub is_bomb_bay:                Vec<bool>,
    pub wpn_type:                   Vec<usize>,
    pub attack_type:                Vec<AttackType>,
    pub target_id:                  Vec<Option<usize>>,
    pub unit_id:                    Vec<usize>,
    pub anim:                       Vec<usize>,
    // ANGLES
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub lock_offset:                Vec<Angle>,
    pub firing_arc:                 Vec<Angle>,
    // Range to start firing or unloading bombs
    pub range:                      Vec<f32>,
    // Length of barrel, or offsetwhere bomb will be launched to.
    pub firing_offset:              Vec<f32>,
    pub fire_rate:                  Vec<f32>,
    pub cooldown:                   Vec<f32>,
    pub salvo:                      Vec<usize>,
    pub salvo_count:                Vec<usize>,
    pub salvo_fire_rate:            Vec<f32>,
    pub salvo_cooldown:             Vec<f32>,
}

impl Weapons {
    pub fn new(num: usize) -> Weapons {
        let mut available_ids = VecDeque::with_capacity(num);
        let mut c: usize = num;

        while c > 0 {
            c -= 1;
            available_ids.push_front(c);
        }

        Weapons {
            available_ids:          available_ids,
            is_bomb_bay:            full_vec(num, false),
            wpn_type:               full_vec(num, 0),
            attack_type:            full_vec(num, AttackType::MeleeAttack(0.0)),
            target_id:              full_vec(num, None),
            unit_id:                full_vec(num, 0),
            anim:                   full_vec(num, 0),
            facing:                 full_vec(num, normalize(0.0)),
            turn_rate:              full_vec(num, normalize(0.0)),
            lock_offset:            full_vec(num, normalize(0.0)),
            firing_arc:             full_vec(num, normalize(0.0)),
            range:                  full_vec(num, 0.0),
            firing_offset:          full_vec(num, 0.0),
            fire_rate:              full_vec(num, 0.0),
            cooldown:               full_vec(num, 0.0),
            salvo:                  full_vec(num, 0),
            salvo_count:            full_vec(num, 0),
            salvo_fire_rate:        full_vec(num, 0.0),
            salvo_cooldown:         full_vec(num, 0.0),
        }
    }

    pub fn make_weapon(&mut self, proto: &Weapon, unit_id: usize) -> usize {
        match self.available_ids.pop_front() {
            Some(id) => {
                self.wpn_type[id]           = proto.wpn_type;
                self.target_id[id]          = None;
                self.unit_id[id]            = unit_id;
                self.anim[id]               = 0;
                self.turn_rate[id]          = proto.turn_rate;
                self.lock_offset[id]        = proto.lock_offset;
                self.firing_arc[id]         = proto.firing_arc;
                self.range[id]              = proto.range;
                self.firing_offset[id]      = proto.firing_offset;
                self.fire_rate[id]          = proto.fire_rate;
                self.cooldown[id]           = 0.0;
                self.salvo[id]              = 0;
                self.salvo_count[id]        = proto.salvo_count;
                self.salvo_fire_rate[id]    = proto.salvo_fire_rate;
                self.salvo_cooldown[id]     = 0.0;
                id
            }
            None => panic!("make_weapon: Not enough weapons to go around.")
        }
    }
}

/*
pub fn destroy_weapon(game: &mut Game, wpn_id: usize) {

}
*/