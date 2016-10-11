use data::aliases::*;
use data::units::UnitTarget;
use std::collections::HashMap;

#[derive(Copy,Clone,Debug)]
pub struct BuildGroupID(usize);

pub struct BuildGroups {
    next_id: usize,
    map: HashMap<usize,BuildGroup>
}

#[derive(Copy,Clone,Debug)]
struct BuildGroup {
    size: usize,
    build_type: UnitTypeID,
    build_target: BuildTarget,
}

#[derive(Clone,Copy,Debug)]
pub enum BuildTarget {
    Point((f32,f32)),
    Unit(UnitTarget),
}

impl BuildGroups {

    pub fn new() -> BuildGroups {
        BuildGroups
        { next_id: 0
        , map: HashMap::new()
        }
    }

    pub fn make_group(&mut self, size: usize, build_type: UnitTypeID, xy: (f32,f32)) -> BuildGroupID {
        let id = self.next_id;
        let bld_group = BuildGroup {
            build_target: BuildTarget::Point(xy),
            build_type: build_type,
            size: size,
        };
        self.map.insert(id, bld_group);
        self.next_id += 1;
        BuildGroupID(id)
    }

    pub fn set_build_target(&mut self, BuildGroupID(bg_id): BuildGroupID, target: UnitTarget) {
        match self.map.get_mut(&bg_id) {
            Some(bg) => {
                bg.build_target = BuildTarget::Unit(target);
            }
            None => {
                panic!("set_build_target: Build group doesn't exist.")
            }
        }
    }

    pub fn build_type(&self, BuildGroupID(bg_id): BuildGroupID) -> UnitTypeID {
        match self.map.get(&bg_id) {
            Some(bg) => {
                bg.build_type
            }
            None => {
                panic!("build_target: Build group doesn't exist.")
            }
        }
    }

    pub fn build_target(&self, BuildGroupID(bg_id): BuildGroupID) -> BuildTarget {
        match self.map.get(&bg_id) {
            Some(bg) => {
                bg.build_target
            }
            None => {
                panic!("build_target: Build group doesn't exist.")
            }
        }
    }

    pub fn done_building(&mut self, BuildGroupID(bg_id): BuildGroupID) {
        let group_is_empty = match self.map.get_mut(&bg_id) {
            Some(bg) => {
                bg.size -= 1;
                bg.size == 0
            }
            None => {
                println!("done_building: Build group doesn't exist.");
                false
            }
        };

        if group_is_empty {
            self.map.remove(&bg_id);
        }
    }
}