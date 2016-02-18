use data::units::{Units};
use data::missiles::{Missiles};
use kdt::{KDTree,Dimensions};
use movement::{Collider};
use data::aliases::*;

#[derive(Clone,Copy)]
pub struct KDTUnit {
    pub id:             UnitID,
    pub team:           TeamID,
    pub x:              f32,
    pub y:              f32,
    pub radius:         f32,
    pub weight:         f32,
    pub target_type:    TargetType,
    pub moving:         bool,
}

impl Dimensions for KDTUnit {
    fn bucket_size() -> usize {256}
    fn num_dims() -> usize {2}
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

impl Collider for KDTUnit {
    fn x_y_radius_weight(&self) -> (f32,f32,f32,f32) {
        (self.x, self.y, self.radius, self.weight)
    }
}

pub fn populate_with_kdtunits(units: &Units) -> KDTree<KDTUnit> {
    let mut vec = Vec::new();

    for id in units.iter() {
        let par = KDTUnit{ id: id
                          , team: units.team[id]
                          , x: units.x[id]
                          , y: units.y[id]
                          , radius: units.radius[id]
                          , weight: units.weight[id]
                          , target_type: units.target_type[id]
                          , moving: units.speed[id] > 0.0};
            vec.push(par);
    }

    KDTree::new(vec)
}

#[derive(Clone,Copy)]
pub struct KDTMissile {
    pub id:             MissileID,
    pub x:              f32,
    pub y:              f32,
}

impl Dimensions for KDTMissile {
    fn bucket_size() -> usize {256}
    fn num_dims() -> usize {2}
    fn dimensions(&self, dim: usize) -> f32 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f32 {
        0.0
    }
}

pub fn populate_with_kdtmissiles(missiles: &Missiles) -> KDTree<KDTMissile> {
    let mut vec = Vec::new();

    for id in missiles.iter() {
        let par = KDTMissile
                { id: id
                , x: missiles.x[id]
                , y: missiles.y[id]
                };
        vec.push(par);
    }

    KDTree::new(vec)
}