use std::collections::vec_deque::{VecDeque};
use movement::{Angle,normalize};
use useful_bits::{full_vec};
use data::aliases::*;

pub struct Weapon {
    pub is_bomb_bay:                    bool,
    pub wpn_type:                       WeaponTypeID,
    pub attack_type:                    AttackType,
    pub turn_rate:                      Angle,
    pub lock_offset:                    Angle,
    pub firing_arc:                     Angle,
    pub range:                          f32,
    pub firing_offset:                  f32,
    pub fire_rate:                      f32,
    pub salvo_count:                    usize,
    pub salvo_fire_rate:                f32,
    pub hits_air:                       bool,
    pub hits_ground:                    bool,
    pub hits_structures:                bool,
}

pub struct Weapons {
    pub available_ids:              VecDeque<WeaponID>,
    // IDENTITY
    pub is_active:                  VecUID<WeaponID,bool>,
    pub is_bomb_bay:                VecUID<WeaponID,bool>,
    pub wpn_type:                   VecUID<WeaponID,WeaponTypeID>,
    pub attack_type:                VecUID<WeaponID,AttackType>,
    pub target_id:                  VecUID<WeaponID,Option<UnitID>>,
    pub unit_id:                    VecUID<WeaponID,UnitID>,
    pub anim:                       VecUID<WeaponID,usize>,
    // ANGLES
    pub facing:                     VecUID<WeaponID,Angle>,
    pub turn_rate:                  VecUID<WeaponID,Angle>,
    pub lock_offset:                VecUID<WeaponID,Angle>,
    pub firing_arc:                 VecUID<WeaponID,Angle>,
    // Range to start firing or unloading bombs
    pub range:                      VecUID<WeaponID,f32>,
    // Length of barrel, or offset where bomb will be launched to.
    pub firing_offset:              VecUID<WeaponID,f32>,
    pub fire_rate:                  VecUID<WeaponID,f32>,
    pub cooldown:                   VecUID<WeaponID,f32>,
    pub salvo:                      VecUID<WeaponID,usize>,
    pub salvo_count:                VecUID<WeaponID,usize>,
    pub salvo_fire_rate:            VecUID<WeaponID,f32>,
    pub salvo_cooldown:             VecUID<WeaponID,f32>,
    // Conditions
    pub hits_air:                   VecUID<WeaponID,bool>,
    pub hits_ground:                VecUID<WeaponID,bool>,
    pub hits_structures:            VecUID<WeaponID,bool>,
}

impl Weapons {
    pub fn new(num: usize) -> Weapons {
        let mut available_ids = VecDeque::with_capacity(num);
        let mut c: usize = num;

        while c > 0 {
            c -= 1;
            available_ids.push_front(WeaponID::unsafe_wrap(c));
        }

        Weapons {
            available_ids:          available_ids,
            is_active:              VecUID::full_vec(num, false),
            is_bomb_bay:            VecUID::full_vec(num, false),
            wpn_type:               VecUID::full_vec(num, 0),
            attack_type:            VecUID::full_vec(num, AttackType::MeleeAttack(0.0)),
            target_id:              VecUID::full_vec(num, None),
            unit_id:                VecUID::full_vec(num, UnitID::unsafe_wrap(0)),
            anim:                   VecUID::full_vec(num, 0),
            facing:                 VecUID::full_vec(num, normalize(0.0)),
            turn_rate:              VecUID::full_vec(num, normalize(0.0)),
            lock_offset:            VecUID::full_vec(num, normalize(0.0)),
            firing_arc:             VecUID::full_vec(num, normalize(0.0)),
            range:                  VecUID::full_vec(num, 0.0),
            firing_offset:          VecUID::full_vec(num, 0.0),
            fire_rate:              VecUID::full_vec(num, 0.0),
            cooldown:               VecUID::full_vec(num, 0.0),
            salvo:                  VecUID::full_vec(num, 0),
            salvo_count:            VecUID::full_vec(num, 0),
            salvo_fire_rate:        VecUID::full_vec(num, 0.0),
            salvo_cooldown:         VecUID::full_vec(num, 0.0),
            hits_air:               VecUID::full_vec(num, false),
            hits_ground:            VecUID::full_vec(num, false),
            hits_structures:        VecUID::full_vec(num, false),
        }
    }

    pub fn make_weapon(&mut self, proto: &Weapon, unit_id: UnitID) -> WeaponID {
        match self.available_ids.pop_front() {
            Some(id) => {
                self.is_active[id]          = true;
                self.is_bomb_bay[id]        = proto.is_bomb_bay;
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
pub fn destroy_weapon(game: &mut Game, wpn_id: WeaponID) {

}
*/