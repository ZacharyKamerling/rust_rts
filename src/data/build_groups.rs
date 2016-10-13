use data::aliases::*;
use data::units::UnitTarget;
use data::aliases::core::cell::Cell;

#[derive(Clone,Debug)]
pub struct BuildGroup {
    build_type: UnitTypeID,
    build_target: Cell<BuildTarget>,
}

#[derive(Clone,Copy,Debug)]
pub enum BuildTarget {
    Point((f32,f32)),
    Unit(UnitTarget),
}

impl BuildGroup {

    pub fn new(bld_type: UnitTypeID, target: BuildTarget) -> BuildGroup {
        BuildGroup
        { build_type: bld_type
        , build_target: Cell::new(target)
        }
    }

    pub fn build_type(&self) -> UnitTypeID {
        self.build_type
    }

    pub fn build_target(&self) -> BuildTarget {
        self.build_target.get()
    }

    pub fn set_build_target(&self, target: BuildTarget) {
        self.build_target.set(target);
    }
}