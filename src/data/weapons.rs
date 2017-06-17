use libs::movement::{Angle, normalize};
use data::aliases::*;
use data::units::UnitTarget;

#[derive(Clone, Copy)]
pub struct ProtoWeapon {
    pub name: &'static str,
    pub attack_type: AttackType,
    pub x_offset: f64,
    pub y_offset: f64,
    pub turn_rate: f64,
    pub lock_offset: Angle,
    pub firing_arc: f64,
    pub missile_speed: f64,
    pub range: f64,
    pub firing_offset: f64,
    pub fire_rate: Milliseconds,
    pub salvo_size: usize,
    pub salvo_fire_rate: Milliseconds,
    pub pellet_count: usize,
    pub pellet_spacing: f64,
    pub random_offset: f64,
    pub target_type: TargetType,
}

pub struct Weapons {
    available_ids: UIDPool<WeaponID>,
    prototypes: VecUID<WeaponTypeID, ProtoWeapon>,
    // IDENTITY
    pub unit_id: VecUID<WeaponID, UnitID>,
    pub wpn_type: VecUID<WeaponID, WeaponTypeID>,
    pub attack_type: VecUID<WeaponID, AttackType>,
    pub target_id: VecUID<WeaponID, Option<UnitTarget>>,
    // Keeps increasing as the weapon goes through a salvo, then resets to 0 at the end.
    // This is useful for animating the weapon as the client can know what stage its in.
    pub anim: VecUID<WeaponID, usize>,
    // Position that the gun is offset on the unit.
    pub xy_offset: VecUID<WeaponID, (f64, f64)>,
    pub facing: VecUID<WeaponID, Angle>,
    pub turn_rate: VecUID<WeaponID, f64>,
    // The angle that represents the center of the units firing arc (relative to the unit its attached to)
    pub lock_offset: VecUID<WeaponID, Angle>,
    // The cone that a weapon can operate in (relative to the unit its attached to)
    pub firing_arc: VecUID<WeaponID, f64>,
    pub missile_speed: VecUID<WeaponID, f64>,
    // Range to start firing or unloading bombs
    pub range: VecUID<WeaponID, f64>,
    // Length of barrel, or offset where bomb will be launched to.
    pub firing_offset: VecUID<WeaponID, f64>,
    // Time between attacks/salvos (AKA attack speed)
    pub fire_rate: VecUID<WeaponID, Milliseconds>,
    // The time until you can launch your next salvo
    pub cooldown: VecUID<WeaponID, Milliseconds>,
    pub salvo_size: VecUID<WeaponID, usize>,
    // The current number of projectiles that have been fired in any given salvo
    pub salvo: VecUID<WeaponID, usize>,
    // The time between each projectile being launched in a salvo
    pub salvo_fire_rate: VecUID<WeaponID, Milliseconds>,
    // The time until you can launch your next projectile(s)
    pub salvo_cooldown: VecUID<WeaponID, Milliseconds>,
    // When you shoot a missile, you can actually shoot more than 1!
    pub pellet_count: VecUID<WeaponID, usize>,
    // The space inbetween each pellet (spread is perpendicular to barrells)
    pub pellet_spacing: VecUID<WeaponID, f64>,
    // Use as a percentage. 10% represents the idea that the projectile can
    // land up to 10% of the distance traveled from its intended target.
    pub random_offset: VecUID<WeaponID, f64>,
    // Conditions
    pub target_type: VecUID<WeaponID, TargetType>,
}

impl Weapons {
    pub fn new(num: usize, prototypes: VecUID<WeaponTypeID, ProtoWeapon>) -> Weapons {
        Weapons {
            available_ids: UIDPool::new(num),
            prototypes: prototypes,
            unit_id: VecUID::full_vec(num, unsafe { UnitID::usize_wrap(0) }),
            wpn_type: VecUID::full_vec(num, unsafe { WeaponTypeID::usize_wrap(0) }),
            attack_type: VecUID::full_vec(num, AttackType::MeleeAttack(Damage::Single(0.0))),
            target_id: VecUID::full_vec(num, None),
            anim: VecUID::full_vec(num, 0),
            xy_offset: VecUID::full_vec(num, (0.0, 0.0)),
            facing: VecUID::full_vec(num, normalize(0.0)),
            turn_rate: VecUID::full_vec(num, 0.0),
            lock_offset: VecUID::full_vec(num, normalize(0.0)),
            firing_arc: VecUID::full_vec(num, 0.0),
            missile_speed: VecUID::full_vec(num, 0.0),
            range: VecUID::full_vec(num, 0.0),
            firing_offset: VecUID::full_vec(num, 0.0),
            fire_rate: VecUID::full_vec(num, 0),
            cooldown: VecUID::full_vec(num, 0),
            salvo_size: VecUID::full_vec(num, 0),
            salvo: VecUID::full_vec(num, 0),
            salvo_fire_rate: VecUID::full_vec(num, 0),
            salvo_cooldown: VecUID::full_vec(num, 0),
            pellet_count: VecUID::full_vec(num, 0),
            pellet_spacing: VecUID::full_vec(num, 0.0),
            random_offset: VecUID::full_vec(num, 0.0),
            target_type: VecUID::full_vec(num, TargetType::new()),
        }
    }

    pub fn make_weapon(&mut self, weapon_type: WeaponTypeID, unit_id: UnitID) -> WeaponID {
        let fps = FPS as f64;
        match self.available_ids.get_id() {
            Some(id) => {
                let proto = &self.prototypes[weapon_type];

                self.unit_id[id] = unit_id;
                self.wpn_type[id] = weapon_type;
                self.attack_type[id] = proto.attack_type;
                self.target_id[id] = None;
                self.anim[id] = 0;
                self.xy_offset[id] = (proto.x_offset, proto.y_offset);
                self.facing[id] = proto.lock_offset;
                self.turn_rate[id] = proto.turn_rate / fps;
                self.lock_offset[id] = proto.lock_offset;
                self.firing_arc[id] = proto.firing_arc;
                self.missile_speed[id] = proto.missile_speed / fps;
                self.range[id] = proto.range;
                self.firing_offset[id] = proto.firing_offset;
                self.fire_rate[id] = proto.fire_rate;
                self.cooldown[id] = 0;
                self.salvo_size[id] = proto.salvo_size;
                self.salvo[id] = 0;
                self.salvo_fire_rate[id] = proto.salvo_fire_rate;
                self.salvo_cooldown[id] = 0;
                self.pellet_count[id] = proto.pellet_count;
                self.pellet_spacing[id] = proto.pellet_spacing;
                self.random_offset[id] = proto.random_offset;
                self.target_type[id] = proto.target_type;
                id
            }
            None => panic!("make_weapon: Not enough weapons to go around."),
        }
    }

    pub fn kill_weapon(&mut self, wpn_id: WeaponID) {
        self.available_ids.put_id(wpn_id);
    }

    pub fn iter(&self) -> Vec<WeaponID> {
        self.available_ids.iter()
    }
}
