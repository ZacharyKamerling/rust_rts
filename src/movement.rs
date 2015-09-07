use std::f64;
use std::f64::consts::{PI};
use std::ops::{Add,Sub};

pub fn dist_to_stop(mut speed: f64, deceleration: f64) -> f64 {
    let mut c = 0.0;
    while speed > 0.0 {
        c += speed;
        speed -= deceleration
    }
    c
}

pub trait Collider {
    fn can_collide(&self, other: &Self) -> bool;
    fn x_y_radius_weight(&self) -> (f64,f64,f64,f64);
}

pub fn collide<T: Collider>(a: T, vec: Vec<T>) -> (f64,f64) {
    let mut xo = 0.0;
    let mut yo = 0.0;
    let (ax,ay,ar,aw) = a.x_y_radius_weight();

    for b in vec.iter() {
        if a.can_collide(b) {
            let (bx,by,br,bw) = a.x_y_radius_weight();
            let x_dif = ax - bx;
            let y_dif = ay - by;
            let r_dif = (ar + br) - f64::sqrt(x_dif * x_dif + y_dif * y_dif);
            let w_dif = (aw + bw) / (2.0 * aw);
            let angl  = f64::atan2(y_dif, x_dif);
            xo += f64::cos(angl) * r_dif * w_dif;
            yo += f64::sin(angl) * r_dif * w_dif
        }
    }
    (xo,yo)
}

#[derive(Clone, Copy, Debug)]
pub struct Angle(f64);

pub fn new(x: f64, y: f64) -> Angle {
    let ang = f64::atan2(y, x);
    if ang < 0.0 {
        Angle(ang + PI * 2.0)
    }
    else {
        Angle(ang)
    }
}

pub fn make_from_unsafe(f: f64) -> Angle {
    Angle(f)
}

pub fn normalize(mut f: f64) -> Angle {
    while f > PI * 2.0 {
        f -= PI * 2.0;
    }
    while f < 0.0 {
        f += PI * 2.0;
    }
    Angle(f)
}

pub fn distance(Angle(a): Angle, Angle(b): Angle) -> f64 {
    let dists = (a - b).abs();

    if dists > PI {
        2.0 * PI - dists
    }
    else {
        dists
    }
}

// Angle to turn, angle to turn towards, amount to turn
pub fn turn_towards(start: Angle, goal: Angle, Angle(turn): Angle) -> Angle {
    let Angle(a) = start;
    let Angle(b) = goal;
    let dist = distance(start, goal);
    let dif = PI - b;
    if a + dif > PI {
        if turn > dist {
            normalize(a - dist)
        }
        else {
            normalize(a - turn)
        }
    }
    else {
        if turn > dist {
            normalize(a + dist)
        }
        else {
            normalize(a + turn)
        }
    }
}

// Angle to lock, Angle being locked to, Arc of leeway
pub fn lock_angle(lock: Angle, org: Angle, Angle(arc): Angle) -> Angle {
    let dist = distance(lock, org);

    if dist > arc {
        turn_towards(lock, org, Angle(dist - arc))
    }
    else {
        lock
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self: Angle, Angle(b): Angle) -> Angle {
        let Angle(a) = self;
        normalize(a + b)
    }
}

impl Sub for Angle {
    type Output = Angle;
    
    fn sub(self: Angle, Angle(b): Angle) -> Angle {
        let Angle(a) = self;
        normalize(a - b)
    }
}