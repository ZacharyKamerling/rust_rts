use pathing::path_grid::PathGrid;
use data::aliases::*;

#[derive(Clone,Debug)]
pub struct Teams {
    available_ids: UIDPool<TeamID>,
    pub prime: VecUID<TeamID, f64>,
    pub energy: VecUID<TeamID, f64>,
    pub max_prime: VecUID<TeamID, f64>,
    pub max_energy: VecUID<TeamID, f64>,
    pub prime_output: VecUID<TeamID, f64>,
    pub energy_output: VecUID<TeamID, f64>,
    pub prime_drain: VecUID<TeamID, f64>,
    pub energy_drain: VecUID<TeamID, f64>,
    pub jps_grid: VecUID<TeamID, PathGrid>,
    pub visible: VecUID<TeamID, VecUID<UnitID, Visibility>>,
    pub visible_missiles: VecUID<TeamID, VecUID<MissileID, Visibility>>,
    build_power_distribution: VecUID<TeamID, VecUID<UnitID, f64>>,
    train_power_distribution: VecUID<TeamID, VecUID<UnitID, f64>>,
}

impl Teams {
    pub fn new(max_units: usize, max_teams: usize, width: usize, height: usize) -> Teams {
        Teams {
            available_ids: UIDPool::new(max_teams),
            prime: VecUID::full_vec(max_teams, 0.0),
            energy: VecUID::full_vec(max_teams, 0.0),
            max_prime: VecUID::full_vec(max_teams, 0.0),
            max_energy: VecUID::full_vec(max_teams, 0.0),
            prime_output: VecUID::full_vec(max_teams, 0.0),
            energy_output: VecUID::full_vec(max_teams, 0.0),
            prime_drain: VecUID::full_vec(max_teams, 0.0),
            energy_drain: VecUID::full_vec(max_teams, 0.0),
            jps_grid: VecUID::full_vec(max_teams, PathGrid::new(width, height)),
            visible: VecUID::full_vec(max_teams, VecUID::full_vec(max_units, Visibility::new())),
            visible_missiles: VecUID::full_vec(max_teams, VecUID::full_vec(max_units * 4, Visibility::new())),
            build_power_distribution: VecUID::full_vec(max_teams, VecUID::full_vec(max_units, 0.0)),
            train_power_distribution: VecUID::full_vec(max_teams, VecUID::full_vec(max_units, 0.0)),
        }
    }

    pub fn make_team(&mut self) -> Option<TeamID> {
        self.available_ids.get_id()
    }

    pub fn iter(&self) -> Vec<TeamID> {
        self.available_ids.iter()
    }

    pub fn apply_build_power(&mut self, team: TeamID, id: UnitID, build_power: f64) {
        self.build_power_distribution[team][id] += build_power;
    }

    pub fn get_build_power_applications(&mut self, team: TeamID) -> (Vec<(UnitID, f64)>) {
        let mut vec = Vec::new();

        for ix in 0..self.build_power_distribution[team].len() {
            let uid = unsafe { UnitID::usize_wrap(ix) };
            let build_power = self.build_power_distribution[team][uid];
            if build_power > 0.0 {
                vec.push((uid, build_power))
            }

            self.build_power_distribution[team][uid] = 0.0;
        }

        vec
    }

    pub fn apply_train_power(&mut self, team: TeamID, id: UnitID, train_power: f64) {
        self.train_power_distribution[team][id] += train_power;
    }

    pub fn get_train_power_applications(&mut self, team: TeamID) -> (Vec<(UnitID, f64)>) {
        let mut vec = Vec::new();

        for ix in 0..self.train_power_distribution[team].len() {
            let uid = unsafe { UnitID::usize_wrap(ix) };
            let train_power = self.train_power_distribution[team][uid];
            if train_power > 0.0 {
                vec.push((uid, train_power))
            }

            self.train_power_distribution[team][uid] = 0.0;
        }

        vec
    }
}