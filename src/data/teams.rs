use jps::{JumpGrid};
use data::aliases::*;

pub struct Teams {
    available_ids:                  UIDPool<TeamID>,
    pub jps_grid:                   VecUID<TeamID, JumpGrid>,
    pub visible:                    VecUID<TeamID, VecUID<UnitID, bool>>,
    pub visible_missiles:           VecUID<TeamID, VecUID<MissileID, bool>>,
}

impl Teams {
    pub fn new(max_units: usize, max_teams: usize, width: usize, height: usize) -> Teams {
        let available_ids = UIDPool::new(max_teams);

        Teams {
            available_ids:  available_ids,
            jps_grid:           VecUID::full_vec(max_teams, JumpGrid::new(width, height)),
            visible:            VecUID::full_vec(max_teams, VecUID::full_vec(max_units, false)),
            visible_missiles:   VecUID::full_vec(max_teams, VecUID::full_vec(max_units * 4, false)),
        }
    }

    pub fn make_team(&mut self) -> Option<TeamID> {
        self.available_ids.get_id()
    }

    pub fn iter(&self) -> Vec<TeamID>
    {
        self.available_ids.iter()
    }
}