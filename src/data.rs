use std::rc::Rc;
use std::collections::vec_deque::{VecDeque};
use kdt::{KDTree,Dimensions};
use jps::{JumpGrid};
use bytegrid::{ByteGrid};
use movement::{Angle,Collider,normalize};
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
    Move(f32,f32)
}

pub struct Game {
    pub available_ids:              Vec<usize>,
    pub unit_blueprints:            Vec<Unit>,
    pub producer_blueprints:        Vec<Producer>,
    pub units:                      Units,
    pub weapons:                    Weapons,
    pub teams:                      Teams,
    pub kdt:                        KDTree<KDTPoint>,
    pub bytegrid:                   ByteGrid,
}

pub struct Teams {
    pub jps_grid:                  Vec<JumpGrid>,
}

pub struct Units {
    // IDENTITY
    pub active_units:               Vec<usize>,
    pub unit_type:                  Vec<usize>,
    pub team:                       Vec<usize>,
    pub anim:                       Vec<usize>,
    pub alive:                      Vec<bool>,
    pub encoding:                   Vec<Vec<u8>>,
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
    pub path:                       Vec<Vec<(isize,isize)>>,
    pub structure_w:                Vec<isize>,
    pub structure_h:                Vec<isize>,
    // STATS
    pub health:                     Vec<f32>,
    pub health_regen:               Vec<f32>,
    pub max_health:                 Vec<f32>,
    pub progress:                   Vec<f32>,
    pub progress_required:          Vec<f32>,
    // PRODUCTION
    pub build_rate:                 Vec<f32>,
    pub build_range:                Vec<f32>,
    pub build_roster:               Vec<Rc<HashSet<usize>>>,
    pub producer:                   Vec<Option<usize>>,
    // COMBAT ORIENTED
    pub weapons:                    Vec<Vec<usize>>,
    pub orders:                     Vec<VecDeque<Order>>,
    pub passengers:                 Vec<Vec<usize>>,
    pub capacity:                   Vec<usize>,
    pub size:                       Vec<usize>,
    pub active_range:               Vec<f32>,
    // FLAGS
    pub is_flying:                  Vec<bool>,
    pub is_structure:               Vec<bool>,
}

pub struct Weapons {
    pub target_id:                  Vec<Option<usize>>,
    pub unit_id:                    Vec<usize>,
    pub anim:                       Vec<usize>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub lock_offset:                Vec<Angle>,
    pub firing_arc:                 Vec<Angle>,
    pub range:                      Vec<f64>,
}

pub struct Projectiles {
    pub id:                         Vec<usize>,
    pub target:                     Vec<Option<usize>>,
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
    pub unit_id:                    Vec<usize>,
    pub blueprint:                  Vec<Rc<HashSet<usize>>>,
    pub stock:                      Vec<usize>,
    pub max_stock:                  Vec<usize>,
    pub progress:                   Vec<f32>,
    pub cooldown:                   Vec<f32>,
    pub build_rate:                 Vec<f32>,
    pub in_production:              Vec<Option<usize>>,
    pub waypoint:                   Vec<(f32,f32)>,
}

pub struct Producer {
    pub unit_id:                    usize,
    pub blueprint:                  Rc<HashSet<usize>>,
    pub stock:                      usize,
    pub max_stock:                  usize,
    pub cooldown:                   f32,
    pub build_rate:                 f32,
    pub waypoint:                   (f32,f32),
}

pub struct Unit {
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
    pub producers:                  Vec<Producer>,
}

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
    Units {
        active_units:           Vec::with_capacity(num), 
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
        min_speed:              full_vec(num, 0.0), 
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
        producer:               full_vec(num, None), 
        build_rate:             full_vec(num, 0.0), 
        build_range:            full_vec(num, 0.0), 
        build_roster:           full_vec(num, empty_roster.clone()), 
        weapons:                full_vec(num, Vec::new()), 
        passengers:             full_vec(num, Vec::new()), 
        capacity:               full_vec(num, 0), 
        size:                   full_vec(num, 0), 
        is_flying:              full_vec(num, false), 
        is_structure:           full_vec(num, false), 
        active_range:           full_vec(num, 0.0), 
        structure_w:            full_vec(num, 0), 
        structure_h:            full_vec(num, 0),
    }
}

impl Game {
    fn kill_unit(&mut self, id: usize) -> () {
        self.available_ids.push(id);
        self.units.alive[id] = false;
    }

    fn make_unit(&mut self) -> Option<usize> {
        match self.available_ids.pop() {
            Some(id) => {
                self.units.alive[id] = true;
                Some(id)
            }
            None => None
        }
    }
}

pub fn populate_with_point_and_radii(units: &Units) -> KDTree<KDTPoint> {
    let mut vec = Vec::new();
    for id in units.active_units.iter() {
        let id = *id;
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
    pub id:         usize,
    pub x:          f32,
    pub y:          f32,
    pub radius:   f32,
    pub weight:     f32,
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

impl Collider for KDTPoint {
    fn x_y_radius_weight(&self) -> (f32,f32,f32,f32) {
        (self.x, self.y, self.radius, self.weight)
    }
}