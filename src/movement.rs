use std::f32;
use std::f32::consts::{PI};
use std::ops::{Add,Sub};

// Area of hexagon
// 2.5980762113533159402911695122588 * r^2

pub type Point = (f32,f32);

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

pub fn collide<A: Collider>(a: A, vec: Vec<A>) -> Point {
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
pub fn turn_towards(start: Angle, goal: Angle, turn: f32) -> Angle {
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
        turn_towards(lock, org, dist - arc)
    }
    else {
        lock
    }
}

pub fn move_in_direction(x: f32, y: f32, speed: f32, Angle(ang): Angle) -> Point {
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

/*
pub fn intercept_point((ax,ay): Point, (bx,by): Point, (vx,vy): Point, speed: f32) -> Option<Point> {
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

        let min_t = f32::min(solution1, solution2);
        let max_t = f32::max(solution1, solution2);

        time =  if min_t > 0.0 {
                    min_t
                }
                else {
                    max_t
                };

        if time < 0.0 {
            return None;
        }
    }

    println!("Time to intercept: {}", time);

    Some((ax + time * vx, ay + time * vy))
}
*/

pub fn get_offset_position((x,y): Point, Angle(angle): Angle, (x_off, y_off): Point) -> Point {
    let coeff = f32::cos(angle);

    (x + coeff * x_off, y + coeff * y_off)
}

pub fn circle_line_intersection((ax,ay): Point, (bx,by): Point, (cx,cy): Point, radius: f32) -> Option<Point> {

    if point_in_circle((ax,ay),(cx,cy),radius) {
        return Some((ax,ay));
    }

    if ax == bx && ay == by {
        return None;
    }

    let ba_x = bx - ax;
    let ba_y = by - ay;
    let ca_x = cx - ax;
    let ca_y = cy - ay;
    // Distances
    let ab_dist = ba_x * ba_x + ba_y * ba_y;
    let ac_dist = ca_x * ca_x + ca_y * ca_y - radius * radius;
    // Cross product
    let cross_bc = ba_x * ca_x + ba_y * ca_y;

    let cross_ratio = cross_bc / ab_dist;
    let ac_ab_ratio = ac_dist / ab_dist;

    let disc = cross_ratio * cross_ratio - ac_ab_ratio;

    if disc < 0.0 {
        return None;
    }

    let sqrt = f32::sqrt(disc);
    let ab_scaling1 = -cross_ratio + sqrt;
    let ab_scaling2 = -cross_ratio + sqrt;

    let p1 = (ax - ba_x * ab_scaling1, ay - ba_y * ab_scaling1);

    if disc == 0.0 {
        return Some(p1);
    }

    let p2 = (ax - ba_x * ab_scaling2, ay - ba_y * ab_scaling2);

    if dist_between_points_sqrd((ax,ay), p1) < dist_between_points_sqrd((ax,ay), p2) {
        Some(p1)
    }
    else {
        Some(p2)
    }
}

fn dist_between_points_sqrd((ax,ay): Point, (bx,by): Point) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;

    dx * dx + dy * dy
}

fn point_in_circle((ax,ay): Point, (bx,by): Point, r: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;

    dx * dx + dy * dy < r * r
}

pub fn test_circle_line_intersection() {
    println!("{:?}", circle_line_intersection((0.0,0.0), (2.0,0.0), (1.0,1.0), 1.0));
    println!("{:?}", circle_line_intersection((0.0,0.0), (1.5,0.0), (2.0,0.0), 1.0));
}

pub fn intercept_point((ax,ay): Point, (bx,by): Point, (vx,vy): Point, s: f32, fps: f32) -> Option<Point> {
    let ox = ax - bx;
    let oy = ay - by;

    let h1 = vx * vx + vy * vy - s * s;
    let h2 = ox * vx + oy * vy;
    let t: f32;

    if h1 == 0.0 { // problem collapses into a simple linear equation
        t = -(ox * ox + oy * oy) / (2.0 * h2);
    } else { // solve the quadratic equation
        let minus_p_half = -h2 / h1;

        let discriminant = minus_p_half * minus_p_half - (ox * ox + oy * oy) / h1; // term in brackets is h3
        if discriminant < 0.0 { // no (real) solution then...
            return None;
        }

        let root = f32::sqrt(discriminant);

        let t1 = minus_p_half + root;
        let t2 = minus_p_half - root;

        let t_min = f32::min(t1, t2);
        let t_max = f32::max(t1, t2);

        t = if t_min > 0.0 { t_min } else { t_max }; // get the smaller of the two times, unless it's negative

        if t < 0.0 { // we don't want a solution in the past
            return None;
        }
    }

    let off_x = t * vx;
    let off_y = t * vy;

    // calculate the point of interception using the found intercept time and return it
    Some((ax + off_x * fps, ay + off_y * fps))
}