use std::collections::vec_deque::VecDeque;

pub enum Relation {
    Transport,
    Creation,
}

pub enum Order {

}

pub enum UnitType {
    NoType
}

pub struct UnitID(pub usize);
pub struct TeamID(pub usize);
pub struct AnimID(pub usize);

pub struct Units {
    pub next_id:                    VecDeque<UnitID>,
    pub team:                       Vec<TeamID>,
    pub anim:                       Vec<AnimID>,
    pub soul:                       Vec<usize>,
    pub alive:                      Vec<bool>,
    pub x:                          Vec<f32>,
    pub y:                          Vec<f32>,
    pub z:                          Vec<f32>,
    pub speed:                      Vec<f32>,
    pub top_speed:                  Vec<f32>,
    pub base_top_speed:             Vec<f32>,
    pub acceleration:               Vec<f32>,
    pub base_acceleration:          Vec<f32>,
    pub deceleration:               Vec<f32>,
    pub base_deceleration:          Vec<f32>,
    pub facing:                     Vec<f32>,
    pub turn_rate:                  Vec<f32>,
    pub base_turn_rate:             Vec<f32>,
    pub health:                     Vec<f32>,
    pub health_regen:               Vec<f32>,
    pub base_health_regen:          Vec<f32>,
    pub max_health:                 Vec<f32>,
    pub base_max_health:            Vec<f32>,
    pub energy:                     Vec<f32>,
    pub energy_regen:               Vec<f32>,
    pub base_energy_regen:          Vec<f32>,
    pub max_energy:                 Vec<f32>,
    pub base_max_energy:            Vec<f32>,
    pub progress:                   Vec<f32>,
    pub start_progress:             Vec<f32>,
    pub unit_type:                  Vec<UnitType>,
    pub orders:                     Vec<VecDeque<Order>>,
    pub parent_relations:           Vec<Vec<(UnitID,Relation)>>,
    pub child_relations:            Vec<Vec<(UnitID,Relation)>>,
}

pub fn make_units(num: usize) -> Units {
    let mut u = Units 
        { next_id:                  VecDeque::with_capacity(num)
        , team:                     Vec::with_capacity(num)
        , anim:                     Vec::with_capacity(num)
        , soul:                     Vec::with_capacity(num)
        , alive:                    Vec::with_capacity(num)
        , x:                        Vec::with_capacity(num)
        , y:                        Vec::with_capacity(num)
        , z:                        Vec::with_capacity(num)
        , speed:                    Vec::with_capacity(num)
        , top_speed:                Vec::with_capacity(num)
        , base_top_speed:           Vec::with_capacity(num)
        , acceleration:             Vec::with_capacity(num)
        , base_acceleration:        Vec::with_capacity(num)
        , deceleration:             Vec::with_capacity(num)
        , base_deceleration:        Vec::with_capacity(num)
        , facing:                   Vec::with_capacity(num)
        , turn_rate:                Vec::with_capacity(num)
        , base_turn_rate:           Vec::with_capacity(num)
        , health:                   Vec::with_capacity(num)
        , health_regen:             Vec::with_capacity(num)
        , base_health_regen:        Vec::with_capacity(num)
        , max_health:               Vec::with_capacity(num)
        , base_max_health:          Vec::with_capacity(num)
        , energy:                   Vec::with_capacity(num)
        , energy_regen:             Vec::with_capacity(num)
        , base_energy_regen:        Vec::with_capacity(num)
        , max_energy:               Vec::with_capacity(num)
        , base_max_energy:          Vec::with_capacity(num)
        , progress:                 Vec::with_capacity(num)
        , start_progress:           Vec::with_capacity(num)
        , unit_type:                Vec::with_capacity(num)
        , orders:                   Vec::with_capacity(num)
        , parent_relations:         Vec::with_capacity(num)
        , child_relations:          Vec::with_capacity(num)
        };
    for n in 0..num {
        u.next_id.push_back(UnitID(n));
        u.team.push(TeamID(0));
        u.anim.push(AnimID(0));
        u.soul.push(0);
        u.alive.push(false);
        u.x.push(0.0);
        u.y.push(0.0);
        u.z.push(0.0);
        u.speed.push(0.0);
        u.top_speed.push(0.0);
        u.base_top_speed.push(0.0);
        u.acceleration.push(0.0);
        u.base_acceleration.push(0.0);
        u.deceleration.push(0.0);
        u.base_deceleration.push(0.0);
        u.facing.push(0.0);
        u.turn_rate.push(0.0);
        u.base_turn_rate.push(0.0);
        u.health.push(0.0);
        u.health_regen.push(0.0);
        u.base_health_regen.push(0.0);
        u.max_health.push(0.0);
        u.base_max_health.push(0.0);
        u.energy.push(0.0);
        u.energy_regen.push(0.0);
        u.base_energy_regen.push(0.0);
        u.max_energy.push(0.0);
        u.base_max_energy.push(0.0);
        u.progress.push(0.0);
        u.start_progress.push(0.0);
        u.unit_type.push(UnitType::NoType);
        u.orders.push(VecDeque::new());
        u.parent_relations.push(Vec::new());
        u.child_relations.push(Vec::new());
    }
    u
}

impl Units {
    fn kill_unit(&mut self, unit_id: UnitID) -> () {
        let UnitID(id) = unit_id;
        self.next_id.push_back(unit_id);
        self.alive[id] = false;
    }

    fn make_unit(&mut self) -> Option<UnitID> {
        match self.next_id.pop_front() {
            Some(unit_id) => {
                let UnitID(id) = unit_id;
                self.alive[id] = true;
                Some(unit_id)
            }
            None => None
        }
    }
}