use jps::{JumpGrid};
use useful_bits::{full_vec};

pub struct Teams {
    pub jps_grid:                   Vec<JumpGrid>,
    pub visible:                    Vec<Vec<bool>>,
}

impl Teams {
    pub fn new(num: usize, width: usize, height: usize) -> Teams {
        Teams {
            jps_grid:       full_vec(8, JumpGrid::new(width, height)),
            visible:        full_vec(8, full_vec(num, false)),
        }
    }
}