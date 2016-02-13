use jps::{JumpGrid};
use useful_bits::{full_vec};
use data::aliases::*;
use data::units::{UnitID};

pub struct Teams {
    pub jps_grid:                   Vec<JumpGrid>,
    pub visible:                    Vec<VecUID<UnitID,bool>>,
}

impl Teams {
    pub fn new(num: usize, width: usize, height: usize) -> Teams {
        Teams {
            jps_grid:       full_vec(8, JumpGrid::new(width, height)),
            visible:        full_vec(8, VecUID::full_vec(num, false)),
        }
    }
}