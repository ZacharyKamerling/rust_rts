use data::target_type::TargetType;

#[derive(Clone, Copy, Debug)]
pub enum OccupyType {
    Structure(isize, isize),
    Unit(MoveStats),
}

#[derive(Clone, Copy, Debug)]
pub struct MoveStats {
    pub collision_radius: f64,
    pub collision_ratio: f64,
    pub collision_resist: f64,
    pub weight: f64,
    pub top_speed: f64,
    pub acceleration: f64,
    pub deceleration: f64,
    pub turn_rate: f64,
    pub collision_type: TargetType,
}
