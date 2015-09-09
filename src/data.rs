use std::rc::Rc;
use std::collections::vec_deque::{VecDeque};
use kdt::{KDTree,Dimensions};
use jps::{JumpGrid};
use movement::{Angle,normalize};
use std::collections::{HashSet};

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Order {

}

pub type TypeID = usize;
pub type UnitID = usize;
pub type WeaponID = usize;
pub type ProducerID = usize;

pub struct Game {
    pub available_ids:              Vec<UnitID>,
    pub blueprints:                 Vec<UnitBlueprint>,
    pub units:                      Units,
    pub jps:                        JumpGrid,
}

pub struct Units {
    // IDENTITY
    pub active_units:               Vec<UnitID>,
    pub unit_type:                  Vec<TypeID>,
    pub team:                       Vec<usize>,
    pub anim:                       Vec<usize>,
    pub alive:                      Vec<bool>,
    // MOVEMENT
    pub x:                          Vec<f32>,
    pub y:                          Vec<f32>,
    pub radius:                     Vec<f32>,
    pub weight:                     Vec<f32>,
    pub speed:                      Vec<f32>,
    pub min_speed:                  Vec<f32>,
    pub top_speed:                  Vec<f32>,
    pub acceleration:               Vec<f32>,
    pub deceleration:               Vec<f32>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    // STATS
    pub health:                     Vec<f32>,
    pub health_regen:               Vec<f32>,
    pub max_health:                 Vec<f32>,
    pub progress:                   Vec<f32>,
    pub progress_required:          Vec<f32>,
    // PRODUCTION
    pub build_rate:                 Vec<f32>,
    pub build_range:                Vec<f32>,
    pub build_roster:               Vec<Rc<HashSet<TypeID>>>,
    pub producer:                   Vec<Option<ProducerID>>,
    // COMBAT ORIENTED
    pub weapons:                    Vec<Vec<WeaponID>>,
    pub orders:                     Vec<VecDeque<Order>>,
    pub passengers:                 Vec<Vec<UnitID>>,
    pub capacity:                   Vec<usize>,
    pub size:                       Vec<usize>,
}

pub struct Weapons {
    pub target_id:                  Vec<Option<UnitID>>,
    pub unit_id:                    Vec<UnitID>,
    pub anim:                       Vec<usize>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub lock_offset:                Vec<Angle>,
    pub firing_arc:                 Vec<Angle>,
}

pub struct Projectiles {
    pub id:                         Vec<usize>,
    pub target:                     Vec<Option<UnitID>>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub x:                          Vec<f32>,
    pub y:                          Vec<f32>,
    pub speed:                      Vec<f32>,
    pub fuel:                       Vec<f32>,
    pub damage:                     Vec<f32>,
    pub damage_radius:              Vec<f32>,
}

pub struct Producers {
    pub unit_id:                    Vec<UnitID>,
    pub blueprints:                 Vec<Rc<HashSet<TypeID>>>,
    pub stock:                      Vec<usize>,
    pub max_stock:                  Vec<usize>,
    pub progress:                   Vec<f32>,
    pub cooldown:                   Vec<f32>,
    pub build_rate:                 Vec<f32>,
    pub in_production:              Vec<Option<UnitID>>,
    pub waypoint:                   Vec<(f32,f32)>,
}

pub struct UnitBlueprint {
    pub radius:                     f32,
    pub weight:                     f32,
    pub min_speed:                  f32,
    pub top_speed:                  f32,
    pub acceleration:               f32,
    pub deceleration:               f32,
    pub turn_rate:                  Angle,
    pub health_regen:               f32,
    pub max_health:                 f32,
    pub progress_required:          f32,
    pub build_rate:                 f32,
    pub build_range:                f32,
    pub producers:                  Vec<ProducerBlueprint>,
}

pub struct ProducerBlueprint;

pub struct WeaponBlueprint;

fn full_vec<T: Clone>(n: usize, default: T) -> Vec<T> {
    let mut vec = Vec::with_capacity(n);
    for _ in 0..n {
        vec.push(default.clone());
    }
    vec
}

pub fn setup_units(num: usize) -> Units {
    let empty_roster = Rc::new(HashSet::new());
    Units 
        { active_units:             Vec::with_capacity(num)
        , unit_type:                full_vec(num, 0)
        , team:                     full_vec(num, 0)
        , anim:                     full_vec(num, 0)
        , alive:                    full_vec(num, false)
        , x:                        full_vec(num, 0.0)
        , y:                        full_vec(num, 0.0)
        , radius:                   full_vec(num, 0.0)
        , weight:                   full_vec(num, 0.0)
        , speed:                    full_vec(num, 0.0)
        , min_speed:                full_vec(num, 0.0)
        , top_speed:                full_vec(num, 0.0)
        , acceleration:             full_vec(num, 0.0)
        , deceleration:             full_vec(num, 0.0)
        , facing:                   full_vec(num, normalize(0.0))
        , turn_rate:                full_vec(num, normalize(0.0))
        , health:                   full_vec(num, 0.0)
        , health_regen:             full_vec(num, 0.0)
        , max_health:               full_vec(num, 0.0)
        , progress:                 full_vec(num, 0.0)
        , progress_required:        full_vec(num, 0.0)
        , orders:                   full_vec(num, VecDeque::new())
        , producer:                 full_vec(num, None)
        , build_rate:               full_vec(num, 0.0)
        , build_range:              full_vec(num, 0.0)
        , build_roster:             full_vec(num, empty_roster.clone())
        , weapons:                  full_vec(num, Vec::new())
        , passengers:               full_vec(num, Vec::new())
        , capacity:                 full_vec(num, 0)
        , size:                     full_vec(num, 0)
        }
}

impl Game {
    fn kill_unit(&mut self, id: UnitID) -> () {
        self.available_ids.push(id);
        self.units.alive[id] = false;
    }

    fn make_unit(&mut self) -> Option<UnitID> {
        match self.available_ids.pop() {
            Some(id) => {
                self.units.alive[id] = true;
                Some(id)
            }
            None => None
        }
    }
}

pub fn populate_with_point_and_radii(units: Units) -> KDTree<KDTPoint> {
    let mut vec = Vec::new();
    for id in units.active_units {
        let par = KDTPoint{ id: id
                          , x: units.x[id]
                          , y: units.y[id]
                          , radius: units.radius[id]
                          , weight: units.weight[id]};
        vec.push(par);
    }
    KDTree::new(vec)
}

#[derive(Clone)]
pub struct KDTPoint {
    id: usize,
    x: f32,
    y: f32,
    radius: f32,
    weight: f32,
}

impl Dimensions for KDTPoint {
    fn num_dims() -> usize {
        2
    }
    fn dimensions(&self, dim: usize) -> f32 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f32 {
        self.radius
    }
}