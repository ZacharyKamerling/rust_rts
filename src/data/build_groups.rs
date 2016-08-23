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
    Point(isize,isize),
    Unit(UnitTarget),
}

impl BuildGroups {

    pub fn new() -> BuildGroups {
        BuildGroups
        { next_id: 0
        , map: HashMap::new()
        }
    }

    pub fn make_group(&mut self, size: usize, build_type: UnitTypeID, x: isize, y: isize) -> BuildGroupID {
        let id = self.next_id;
        let bld_group = BuildGroup {
            build_target: BuildTarget::Point(x,y),
            build_type: build_type,
            size: size,
        };
        self.map.insert(id, bld_group);
        self.next_id += 1;
        BuildGroupID(id)
    }

    pub fn build_target(&self, BuildGroupID(bg_id): BuildGroupID) -> BuildTarget {
        match self.map.get(&bg_id) {
            Some(&bg) => {
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