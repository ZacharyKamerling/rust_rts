extern crate time;

use self::time::{PreciseTime};
use std::f64::consts::{PI};
use std::ops::{Add,Sub,Mul,Div};
use core::ops::Deref;

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct FixedAngle {
    data: u16,
}

impl Add for FixedAngle {
    type Output = FixedAngle;
    fn add(self, other: FixedAngle) -> FixedAngle {
        FixedAngle { data: self.data + other.data }
    }
}

impl Sub for FixedAngle {
    type Output = FixedAngle;
    fn sub(self, other: FixedAngle) -> FixedAngle {
        FixedAngle { data: self.data - other.data }
    }
}

impl FixedAngle {

    pub fn to_f64(&self) -> f64 {
        self.data as f64 * PI * 2.0 / u16::max_value() as f64
    }

    pub fn distance(self, other: FixedAngle) -> FixedAngle {
        let a = self.data;
        let b = other.data;

        let dists = {
            if a > b {
                a - b
            }
            else {
                b - a
            }
        };

        if dists > u16::max_value() / 2 {
            FixedAngle { data: u16::max_value() - dists }
        }
        else {
            FixedAngle { data: dists }
        }
    }

    pub fn turn_towards(self, goal: FixedAngle, amount: FixedAngle) -> FixedAngle {
        let a = self.data;
        let b = goal.data;
        let turn = amount.data;
        let dist = self.distance(goal).data;

        if turn > dist {
            goal
        }
        else {
            if a > b {
                if a - b > u16::max_value() / 2 {
                    FixedAngle { data: a + turn }
                }
                else {
                    FixedAngle { data: a - turn }
                }
            }
            else {
                if b - a > u16::max_value() / 2 {
                    FixedAngle { data: a - turn }
                }
                else {
                    FixedAngle { data: a + turn }
                }
            }
        }
    }

    // Angle to lock, Angle being locked to, Arc of leeway
    pub fn lock_angle(self: FixedAngle, org: FixedAngle, arc: FixedAngle) -> FixedAngle {
        let dist = self.distance(org);

        if dist > arc {
            self.turn_towards(org, dist - arc)
        }
        else {
            self
        }
    }
}

pub fn dist_to_stop(mut speed: UFixed64, deceleration: UFixed64) -> UFixed64 {
    let mut c = UFixed64 { data: 0 };
    while speed.data > 0 {
        c = c + speed;
        speed = speed - deceleration;
    }
    c
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct UFixed64 {
    data: u64,
}

impl Add for UFixed64 {
    type Output = UFixed64;
    fn add(self, other: UFixed64) -> UFixed64 {
        UFixed64 { data: self.data + other.data }
    }
}

impl Sub for UFixed64 {
    type Output = UFixed64;
    fn sub(self, other: UFixed64) -> UFixed64 {
        UFixed64 { data: self.data - other.data }
    }
}

impl Mul for UFixed64 {
    type Output = UFixed64;
    fn mul(self, other: UFixed64) -> UFixed64 {
        UFixed64 { data: self.data * other.data }
    }
}

impl Div for UFixed64 {
    type Output = UFixed64;
    fn div(self, other: UFixed64) -> UFixed64 {
        UFixed64 { data: self.data / other.data }
    }
}

impl UFixed64 {
    pub fn new(a: u32) -> UFixed64 {
        UFixed64 { data: a as u64 * 4294967296 }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct Coord {
    data: i32, // Can represent [-1023,1023] with 1048576 spaces between each number
}

impl Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord { data: self.data + other.data }
    }
}

impl Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord { data: self.data - other.data }
    }
}

impl Mul for Coord {
    type Output = Coord;
    fn mul(self, other: Coord) -> Coord {
        Coord { data: self.data * other.data }
    }
}

impl Div for Coord {
    type Output = Coord;
    fn div(self, other: Coord) -> Coord {
        Coord { data: self.data / other.data }
    }
}

impl Coord {
    pub fn new(f: f64) -> Coord {
        let granularity = (i32::max_value() / 2 + 1) as f64 / 1024.0; // 1048576

        if f > 1023.0 || f < -1023.0 {
            panic!("Tried to create a Coord with value GT 1024 or LT -1024.");
        }

        let val = (f * granularity) as i32;
        Coord { data: val }
    }

    pub fn to_f64(self) -> f64 {
        self.data as f64 / 1048576.0
    }
}

pub trait Collider {
    fn x_y_radius_weight(&self) -> (Coord,Coord,Coord,Coord);
}

pub struct FixedBase {
    cos_sin: Vec<(i32,i32)>,
    atan2: Vec<u16>,
}

impl FixedBase {
    pub fn new() -> FixedBase {
        let mut cos_sin = Vec::with_capacity(u16::max_value() as usize);
        let mut atan2 = Vec::with_capacity(1024 * 1024);

        let max_i32 = i32::max_value() as f64;

        let ix = PI * 2.0;
        for i in 0..u16::max_value() {
            let f = i as f64 / u16::max_value() as f64 * ix;
            let mut xo = f64::cos(f);
            let mut yo = f64::sin(f);

            cos_sin.push(((xo * max_i32) as i32, (yo * max_i32) as i32));
        }

        let qrtr_max_u16 = u16::max_value() as f64 / 4.0;
        for y in 0..1024 {
            for x in 0..1024 {
                let angl = (f64::atan2(y as f64, x as f64) / (PI / 2.0) * qrtr_max_u16) as u16;
                atan2.push(angl);
            }
        }

        FixedBase {
            cos_sin: cos_sin,
            atan2: atan2,
        }
    }

    #[inline(always)]
    pub fn atan2(&self, ffy: Coord, ffx: Coord) -> FixedAngle {
        let x = ffx.data;
        let y = ffy.data;

        if x >= 0 {
            if y >= 0 {
                // NE
                FixedAngle { data: self.qrtr_atan2(y, x) }
            }
            else {
                // SE
                FixedAngle { data: u16::max_value() - self.qrtr_atan2(y * -1, x) }
            }
        }
        else {
            // NW
            if y >= 0 {
                FixedAngle { data: u16::max_value() / 2 - self.qrtr_atan2(y, x * -1) }
            }
            else {
            //SW
                FixedAngle { data: u16::max_value() / 2 + self.qrtr_atan2(y * -1, x * -1) }
            }
        }
    }

    #[inline(always)]
    pub fn dist(&self, (ax,ay): (Coord,Coord), (bx,by): (Coord,Coord)) -> f64 {
        let x_dif = (bx - ax).data as i64;
        let y_dif = (by - ay).data as i64;

        f64::sqrt((x_dif * x_dif + y_dif * y_dif) as f64) * 1048576.0
    }

    fn qrtr_atan2(&self, mut y: i32, mut x: i32) -> u16 {
        x = i32::abs(x);
        y = i32::abs(y);
        let d = {
            if x > y {
                x / 1024 + 1
            }
            else {
                y / 1024 + 1
            }
        };

        x /= d;
        y /= d;

        self.atan2[(y * 1024 + x) as usize]
    }

    pub fn offset(&self, a: FixedAngle, offset: Coord) -> (Coord,Coord) {
        let max_i16 = i16::max_value() as i64;
        let (xo,yo) = self.cos_sin[a.data as usize];
        let x = offset.data as i64 * xo as i64 / max_i16;
        let y = offset.data as i64 * yo as i64 / max_i16;

        (Coord {data : x as i32}, Coord {data: y as i32})
    }
}

pub fn bench() {
    let mili = 1000000.0;
    let tb = FixedBase::new();

    for x in 0..1024 {
        for y in 0..1024 {
            let angle1 = f64::atan2(y as f64, x as f64);
            let angle2 = tb.atan2(Coord {data: y as i32}, Coord {data: x as i32});
            /*let dist = tb.dist((Coord::new(0.0), Coord::new(0.0)), (Coord::new(x as f64), Coord::new(y as f64)));
            let dist2 = f64::sqrt((x * x + y * y) as f64);

            if f64::abs(dist2 - dist.to_f64()) > 1.0 {
                println!("Distance Wrong! {:?}, {:?}, {:?} {:?} {:?}", dist, dist.to_f64(), dist2, x, y);
                return;
            }
            */

            if f64::abs(angle2.to_f64() - angle1) > 0.0001 {
                println!("Angle Wrong! {:?} {:?} {:?} {:?} {:?}", angle1, angle2, angle2.to_f64(), x, y);
                return;
            }
        }
    }

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let angl = f32::atan2(y as f32, x as f32);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard atan2 time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let angl = tb.atan2(Coord::new(y as f64), Coord::new(x as f64));
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Coord atan2 time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let xf = x as f32;
            let yf = y as f32;
            let dist = f32::sqrt(xf * xf + yf * yf);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard sqrt time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let zero = Coord { data: 0 };
            let x_coord = Coord { data: x as i32 };
            let y_coord = Coord { data: y as i32 };
            let dist = tb.dist((zero, zero), (x_coord,y_coord));
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Coord sqrt time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let a = x as f64 + y as f64;
            let a = x as f64 - y as f64;
            let a = x as f64 * y as f64;
            let a = x as f64 / (y + 1) as f64;
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard ops time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let a = x + y;
            let a = x - y;
            let a = x * y;
            let a = x / (y + 1);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Coord ops time: {}ms", total as f32 / mili);
}