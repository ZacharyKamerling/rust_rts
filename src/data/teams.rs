use jps::{JumpGrid};
use vis_set::{VisSet};
//use data::aliases::*;
use useful_bits::{full_vec};

pub struct Teams {
    pub jps_grid:                   Vec<JumpGrid>,
    pub visible:                    Vec<VisSet>,
}

impl Teams {
    pub fn new(num: usize, width: usize, height: usize) -> Teams {
        Teams {
            jps_grid:       full_vec(num, JumpGrid::new(width, height)),
            visible:        full_vec(num, VisSet::with_capacity(512)),
        }
    }
}