use jps::{JumpGrid};
use std::collections::HashSet;
//use data::aliases::*;
use useful_bits::{full_vec};

pub struct Teams {
    pub jps_grid:                   Vec<JumpGrid>,
    pub visible:                    Vec<HashSet<usize>>,
}

impl Teams {
    pub fn new(num: usize, width: usize, height: usize) -> Teams {
        Teams {
            jps_grid:       full_vec(num, JumpGrid::new(width, height)),
            visible:        full_vec(num, HashSet::with_capacity(8)),
        }
    }
}