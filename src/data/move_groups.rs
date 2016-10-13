/* As units approach the end of their path, they will bunch up and some of them will never reach their destination.
This is natural, but what is not natural is they will keep trying to reach the end forever.
To prevent this, we put units into a move group. As they reach their destination, the move group records how many
have reached the destination and adds up their total area (plus 25% extra). Units then only have to move within the
circular area to complete their movement.
*/

use std::f32;
use data::aliases::core::cell::Cell;

#[derive(Clone,Debug)]
pub struct MoveGroup {
    area: Cell<f32>,
    dist: Cell<f32>,
    xy: Cell<(f32,f32)>,
}

impl MoveGroup {

    pub fn new(xy: (f32,f32)) -> MoveGroup {
        MoveGroup
        { area: Cell::new(0.0)
        , dist: Cell::new(0.0)
        , xy: Cell::new(xy)
        }
    }

    pub fn done_moving(&self, radius: f32) {
        self.area.set(self.area.get() + radius * radius);
        self.dist.set(f32::sqrt(self.area.get()) * 1.25);
    }

    pub fn dist_to_group(&self) -> f32 {
        self.dist.get()
    }

    pub fn goal(&self) -> (f32,f32) {
        self.xy.get()
    }

    pub fn set_goal(&self, xy: (f32,f32)) {
        self.xy.set(xy);
    }
}