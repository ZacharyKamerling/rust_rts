use std::collections::vec_deque::VecDeque;
use kdt::{KDTree,Dimensions};

pub enum Flag {
    IsUnit,
    IsStructure,
    IsMobile,
    IsGround,
    IsFlying,
    IsMissile,
    IsAutomated,
    IsTransportable,
}

pub enum Relation {
    Transportation,
    Creation,
}

pub enum Order {

}

pub struct TypeID(usize);
pub struct UnitID(usize);
pub struct TeamID(usize);
pub struct AnimID(usize);

pub struct Game {
    pub available_ids:              Vec<UnitID>,
    pub blueprints:                 Vec<Blueprint>,
    pub units:                      Units,
}

pub struct Units {
    pub unit_type:                  Vec<TypeID>,
    pub team:                       Vec<TeamID>,
    pub anim:                       Vec<AnimID>,
    pub alive:                      Vec<bool>,
    pub x:                          Vec<f64>,
    pub y:                          Vec<f64>,
    pub radius:                     Vec<f64>,
    pub weight:                     Vec<f64>,
    pub speed:                      Vec<f64>,
    pub min_speed:                  Vec<f64>,
    pub top_speed:                  Vec<f64>,
    pub acceleration:               Vec<f64>,
    pub deceleration:               Vec<f64>,
    pub facing:                     Vec<f64>,
    pub turn_rate:                  Vec<f64>,
    pub health:                     Vec<f64>,
    pub health_regen:               Vec<f64>,
    pub max_health:                 Vec<f64>,
    pub energy:                     Vec<f64>,
    pub energy_regen:               Vec<f64>,
    pub max_energy:                 Vec<f64>,
    pub progress:                   Vec<f64>,
    pub progress_required:          Vec<f64>,
    pub orders:                     Vec<VecDeque<Order>>,
    pub producers:                  Vec<Vec<Producer>>,
    pub parent_relations:           Vec<Vec<(UnitID,Relation)>>,
    pub child_relations:            Vec<Vec<(UnitID,Relation)>>,
}

pub struct Blueprint {
    pub radius:                     f64,
    pub weight:                     f64,
    pub min_speed:                  f64,
    pub top_speed:                  f64,
    pub acceleration:               f64,
    pub deceleration:               f64,
    pub turn_rate:                  f64,
    pub health_regen:               f64,
    pub max_health:                 f64,
    pub energy_regen:               f64,
    pub max_energy:                 f64,
    pub progress_required:          f64,
    pub producers:                  Vec<Producer>,
}

pub struct Producer {
    pub blueprints:                 Vec<TypeID>,
    pub stock:                      usize,
    pub max_stock:                  usize,
    pub progress:                   f64,
    pub cooldown:                   f64,
    pub build_rate:                 f64,
    pub in_production:              Option<UnitID>,
    pub waypoint:                   (f64,f64),
}

pub fn setup_units(num: usize) -> Units {
    let mut u = Units 
        { unit_type:                Vec::with_capacity(num)
        , team:                     Vec::with_capacity(num)
        , anim:                     Vec::with_capacity(num)
        , alive:                    Vec::with_capacity(num)
        , x:                        Vec::with_capacity(num)
        , y:                        Vec::with_capacity(num)
        , radius:                   Vec::with_capacity(num)
        , weight:                   Vec::with_capacity(num)
        , speed:                    Vec::with_capacity(num)
        , min_speed:                Vec::with_capacity(num)
        , top_speed:                Vec::with_capacity(num)
        , acceleration:             Vec::with_capacity(num)
        , deceleration:             Vec::with_capacity(num)
        , facing:                   Vec::with_capacity(num)
        , turn_rate:                Vec::with_capacity(num)
        , health:                   Vec::with_capacity(num)
        , health_regen:             Vec::with_capacity(num)
        , max_health:               Vec::with_capacity(num)
        , energy:                   Vec::with_capacity(num)
        , energy_regen:             Vec::with_capacity(num)
        , max_energy:               Vec::with_capacity(num)
        , progress:                 Vec::with_capacity(num)
        , progress_required:        Vec::with_capacity(num)
        , orders:                   Vec::with_capacity(num)
        , producers:                Vec::with_capacity(num)
        , parent_relations:         Vec::with_capacity(num)
        , child_relations:          Vec::with_capacity(num)
        };
    for _ in 0..num {
        u.unit_type.push(TypeID(0));
        u.team.push(TeamID(0));
        u.anim.push(AnimID(0));
        u.alive.push(false);
        u.x.push(0.0);
        u.y.push(0.0);
        u.radius.push(0.0);
        u.weight.push(0.0);
        u.speed.push(0.0);
        u.min_speed.push(0.0);
        u.top_speed.push(0.0);
        u.acceleration.push(0.0);
        u.deceleration.push(0.0);
        u.facing.push(0.0);
        u.turn_rate.push(0.0);
        u.health.push(0.0);
        u.health_regen.push(0.0);
        u.max_health.push(0.0);
        u.energy.push(0.0);
        u.energy_regen.push(0.0);
        u.max_energy.push(0.0);
        u.progress.push(0.0);
        u.progress_required.push(0.0);
        u.orders.push(VecDeque::new());
        u.producers.push(Vec::new());
        u.parent_relations.push(Vec::new());
        u.child_relations.push(Vec::new());
    }
    u
}

pub fn populate_with_point_and_radii(units: Units) -> KDTree<KDTPoint> {
    let mut vec = Vec::new();
    for id in 0..units.alive.len() {
        if units.alive[id] {
            let par = KDTPoint{ id: id
                                   , x: units.x[id]
                                   , y: units.y[id]
                                   , radius: units.radius[id]
                                   , weight: units.weight[id]};
            vec.push(par);
        }
    }
    KDTree::new(vec)
}

#[derive(Clone)]
pub struct KDTPoint {
    id: usize,
    x: f64,
    y: f64,
    radius: f64,
    weight: f64,
}

impl Dimensions for KDTPoint {
    fn num_dims() -> usize {
        2
    }
    fn dimensions(&self, dim: usize) -> f64 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f64 {
        self.radius
    }
}

impl Game {
    fn kill_unit(&mut self, unit_id: UnitID) -> () {
        let UnitID(id) = unit_id;
        self.available_ids.push(unit_id);
        self.units.alive[id] = false;
    }

    fn make_unit(&mut self) -> Option<UnitID> {
        match self.available_ids.pop() {
            Some(unit_id) => {
                let UnitID(id) = unit_id;
                self.units.alive[id] = true;
                Some(unit_id)
            }
            None => None
        }
    }
}