use data::units::{Units,UnitID};
use kdt::{KDTree,Dimensions};
use movement::{Collider};

pub fn populate_with_kdtpoints(units: &Units) -> KDTree<KDTPoint> {
    let mut vec = Vec::new();

    for id in units.iter() {
        let par = KDTPoint{ id: id
                          , team: units.team[id]
                          , x: units.x[id]
                          , y: units.y[id]
                          , radius: units.radius[id]
                          , weight: units.weight[id]
                          , flying: units.is_flying[id]
                          , structure: units.is_structure[id]
                          , ground: units.is_ground[id]
                          , moving: units.speed[id] > 0.0};
            vec.push(par);
    }

    KDTree::new(vec)
}

#[derive(Clone,Copy)]
pub struct KDTPoint {
    pub id:         UnitID,
    pub team:       usize,
    pub x:          f32,
    pub y:          f32,
    pub radius:     f32,
    pub weight:     f32,
    pub flying:     bool,
    pub structure:  bool,
    pub ground:     bool,
    pub moving:     bool,
}

impl Dimensions for KDTPoint {
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

impl Collider for KDTPoint {
    fn x_y_radius_weight(&self) -> (f32,f32,f32,f32) {
        (self.x, self.y, self.radius, self.weight)
    }
}