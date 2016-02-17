use std::f32;
use std::f32::consts::{PI};
use std::ops::{Add,Sub};

// Area of hexagon
// 2.5980762113533159402911695122588 * r^2

pub fn dist_to_stop(mut speed: f32, deceleration: f32) -> f32 {
    let mut c = 0.0;
    while speed > 0.0 {
        c += speed;
        speed -= deceleration;
    }
    c
}

pub trait Collider {
    fn x_y_radius_weight(&self) -> (f32,f32,f32,f32);
}

pub fn collide<A: Collider>(a: A, vec: Vec<A>) -> (f32,f32) {
    let mut xo = 0.0;
    let mut yo = 0.0;
    let (ax,ay,ar,aw) = a.x_y_radius_weight();

    for b in vec.iter() {
        let (bx,by,br,bw) = b.x_y_radius_weight();
        let x_dif = ax - bx;
        let y_dif = ay - by;
        let r_dif = (ar + br) - f32::sqrt(x_dif * x_dif + y_dif * y_dif);
        let w_dif = bw / aw;
        let angl  = f32::atan2(y_dif, x_dif);
        xo += f32::cos(angl) * r_dif * w_dif;
        yo += f32::sin(angl) * r_dif * w_dif;
    }
    (xo,yo)
}

#[derive(Clone, Copy, Debug)]
pub struct Angle(f32);

pub fn new(x: f32, y: f32) -> Angle {
    let ang = f32::atan2(y, x);
    if ang < 0.0 {
        Angle(ang + PI * 2.0)
    }
    else {
        Angle(ang)
    }
}

pub unsafe fn make_from(f: f32) -> Angle {
    Angle(f)
}

pub fn normalize(mut f: f32) -> Angle {
    while f > PI * 2.0 {
        f -= PI * 2.0;
    }
    while f < 0.0 {
        f += PI * 2.0;
    }
    Angle(f)
}

pub fn denormalize(Angle(f): Angle) -> f32 { f }

pub fn distance(Angle(a): Angle, Angle(b): Angle) -> f32 {
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

    if turn > dist {
        goal
    }
    else {
        if a > b {
            if a - b > PI {
                normalize(a + turn)
            }
            else {
                normalize(a - turn)
            }
        }
        else {
            if b - a > PI {
                normalize(a - turn)
            }
            else {
                normalize(a + turn)
            }
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

pub fn move_in_direction(x: f32, y: f32, speed: f32, Angle(ang): Angle) -> (f32,f32) {
    (x + f32::cos(ang) * speed, y + f32::sin(ang) * speed)
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

pub fn intercept_point((ax,ay): (f32,f32), (bx,by): (f32,f32), (vx,vy): (f32,f32), speed: f32) -> Option<(f32,f32)> {
    let dx = ax - bx;
    let dy = ay - by;
    let sqrd_dist1 = dx * dx + dy * dy;
    let sqrd_dist2 = vx * vx + vy * vy - speed * speed;
    let sqrd_dist3 = dx * vx + dy * vy;
    let time;

    if sqrd_dist1 == 0.0 {
        time = -sqrd_dist1 / (2.0 * sqrd_dist3);
    }
    else {
        let neg_half = -sqrd_dist3 / sqrd_dist2;
        let discriminant = neg_half * neg_half - sqrd_dist1 / sqrd_dist2;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_discriminant = f32::sqrt(discriminant);
        let solution1 = neg_half + sqrt_discriminant;
        let solution2 = neg_half - sqrt_discriminant;

        if solution1 > 0.0 && solution1 < solution2 {
            time = solution1;
        }
        else if solution2 > 0.0 {
            time = solution2;
        }
        else {
            return None;
        }
    }

    Some((ax + time * vx, ay + time * vy))
}

pub fn get_offset_position((x,y): (f32,f32), angle: Angle, (x_off, y_off): (f32,f32)) -> (f32,f32) {
    let coeff = f32::cos(denormalize(angle));

    (x + coeff * x_off, y + coeff * y_off)
}