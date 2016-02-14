use jps::{JumpGrid};
use useful_bits::{full_vec};
use data::aliases::*;

pub struct Teams {
    pub jps_grid:                   VecUID<TeamID, JumpGrid>,
    pub visible:                    VecUID<TeamID, VecUID<UnitID, bool>>,
}

impl Teams {
    pub fn new(num: usize, width: usize, height: usize) -> Teams {
        Teams {
            jps_grid:       VecUID::full_vec(8, JumpGrid::new(width, height)),
            visible:        VecUID::full_vec(8, VecUID::full_vec(num, false)),
        }
    }
}