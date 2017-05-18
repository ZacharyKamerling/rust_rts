extern crate time;

use self::time::{PreciseTime};
use std::f64::consts::{PI};
use std::ops::{Add,Sub};

pub struct Fast(i64);

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct Angle(u16);

impl Add for Angle {
    type Output = Angle;
    fn add(self, Angle(b): Angle) -> Angle {
        let Angle(a) = self;
        Angle(a + b)
    }
}

impl Sub for Angle {
    type Output = Angle;
    fn sub(self, Angle(b): Angle) -> Angle {
        let Angle(a) = self;
        Angle(a - b)
    }
}

impl Angle {

    pub fn to_f64(self) -> f64 {
        let Angle(a) = self;
        a as f64 * PI * 2.0 / u16::max_value() as f64
    }

    pub fn distance(self, Angle(b): Angle) -> Angle {
        let Angle(a) = self;

        let dists = {
            if a > b {
                a - b
            }
            else {
                b - a
            }
        };

        if dists > u16::max_value() / 2 {
            Angle(u16::max_value() - dists)
        }
        else {
            Angle(dists)
        }
    }

    pub fn turn_towards(self, goal: Angle, Angle(turn): Angle) -> Angle {
        let Angle(a) = self;
        let Angle(b) = goal;
        let Angle(dist) = self.distance(goal);

        if turn > dist {
            goal
        }
        else if a > b {
            if a - b > u16::max_value() / 2 {
                Angle(a + turn)
            }
            else {
                Angle(a - turn)
            }
        }
        else if b - a > u16::max_value() / 2 {
            Angle(a - turn)
        }
        else {
            Angle(a + turn)
        }
    }

    // Angle to lock, Angle being locked to, Arc of leeway
    pub fn lock_angle(self, org: Angle, arc: Angle) -> Angle {
        let dist = self.distance(org);

        if dist > arc {
            self.turn_towards(org, dist - arc)
        }
        else {
            self
        }
    }
}

pub struct FastBase {
    cos_sin: Vec<(i32,i32)>,
    atan2: Vec<u16>,
}

impl FastBase {

    pub fn new() -> FastBase {
        let capacity = u16::max_value() as usize;
        let mut cos_sin = Vec::with_capacity(capacity);
        let mut atan2 = Vec::with_capacity(capacity);

        let f16 = u16::max_value() as f64;

        for i in 0..capacity {
            let f = i as f64 / f16 * PI * 2.0;
            let xo = f64::cos(f);
            let yo = f64::sin(f);

            cos_sin.push(((xo * f16) as i32, (yo * f16) as i32));
        }

        let qrtr_f16 = f16 / 2.0;

        for i in 0..capacity + 1 {
            let f = i as f64 / f16;
            atan2.push((f64::atan2(f, 1.0) / PI * qrtr_f16) as u16);
        }

        FastBase {
            atan2: atan2,
            cos_sin: cos_sin,
        }
    }

    pub fn coord(&self, f: f64) -> Fast {
        Fast((f * 140737488355328u64 as f64 / 65536 as f64) as i64)
    }

    pub fn atan2(&self, Fast(y): Fast, Fast(x): Fast) -> Angle {
        let ax = x.abs();
        let ay = y.abs();

        // Octants
        if x > 0 && y > 0 && ax >= ay { // 0
            let r = self.atan2[self::ratio16(ay,ax) as usize];
            let angle = r;

            return Angle(angle);
        }

        if x > 0 && y > 0 && ay >= ax { // 1
            let angle_offset = self.atan2[self::ratio16(ax,ay) as usize];
            let angle = u16::max_value() / 4 - angle_offset;

            return Angle(angle);
        }

        if x < 0 && y > 0 && ay >= ax { // 2
            let angle_offset = self.atan2[self::ratio16(ax,ay) as usize];
            let angle = u16::max_value() / 4 + angle_offset;

            return Angle(angle);
        }

        if x < 0 && y > 0 && ax >= ay { // 3
            let angle_offset = self.atan2[self::ratio16(ay,ax) as usize];
            let angle = u16::max_value() / 2 - angle_offset;

            return Angle(angle);
        }

        if x < 0 && y < 0 && ax >= ay { // 4
            let angle_offset = self.atan2[self::ratio16(ay,ax) as usize];
            let angle = u16::max_value() / 2 + angle_offset;

            return Angle(angle);
        }

        if x < 0 && y < 0 && ay >= ax { // 5
            let angle_offset = self.atan2[self::ratio16(ax,ay) as usize];
            let angle = u16::max_value() / 4 * 3 - angle_offset;

            return Angle(angle);
        }

        if x > 0 && y < 0 && ay >= ax { //6
            let angle_offset = self.atan2[self::ratio16(ax,ay) as usize];
            let angle = u16::max_value() / 4 * 3 + angle_offset;

            return Angle(angle);
        }

        if x > 0 && y < 0 && ax >= ay { // 7
            let angle_offset = self.atan2[self::ratio16(ay,ax) as usize];
            let angle = u16::max_value() - angle_offset;

            return Angle(angle);
        }

        //CORNER CASES
        if x == 0 && y == 0 {
            return Angle(0);
        }

        if x == 0 {
            if y > 0 {
                return Angle(u16::max_value() / 4);
            }
            else {
                return Angle(u16::max_value() - u16::max_value() / 4);
            }
        }

        if y == 0 {
            if x > 0 {
                return Angle(0);
            }
            else {
                return Angle(u16::max_value() / 2);
            }
        }

        Angle(0)
    }

    pub fn dist(&self, (Fast(x),Fast(y)): (Fast,Fast)) -> Fast {
        /*
        x & y are 47 bits. We need to reduce them to 31 bits before squaring & adding them (to total no more than 63 bits).
        */
        let denom = 65536 as i64;
        let x = x / denom;
        let y = y / denom;

        Fast(f64::sqrt((x * x + y * y) as f64) as i64 * denom)
    }

    pub fn in_range(&self, (Fast(x),Fast(y)): (Fast,Fast), Fast(r): Fast) -> bool {
        /*
        x & y are 47 bits. We need to reduce them to 31 bits before squaring & adding them (to total no more than 63 bits).
        */
        let denom = 65536 as i64;
        let x = x / denom;
        let y = y / denom;
        let r = r / denom;

        x * x + y * y < r * r
    }

    pub fn offset(&self, Angle(a): Angle, Fast(f): Fast) -> (Fast,Fast) {
        let (xo,yo) = self.cos_sin[a as usize]; //X & Y offsets
        let denom = 65536 as i64;
        let fx = Fast((xo as i64) * f / denom);
        let fy = Fast((yo as i64) * f / denom);

        (fx,fy)
    }
}

fn ratio16(mut a: i64, mut b: i64) -> u16 {
    let denom = 65536 as i64;
    a = i64::abs(a);
    b = i64::abs(b);

    if a == b {
        return (denom - 1) as u16;
    }

    (a * denom / b) as u16
}

pub fn bench() {
    let mili = 1000000.0;
    let fb = FastBase::new();

    for x in 0..1024 {
        for y in 0..1024 {
            let angle1 = f64::atan2(y as f64, x as f64);
            let angle2 = fb.atan2(Fast(y), Fast(x));

            if f64::abs(angle2.to_f64() - angle1) > 0.001 {
                println!("Angle Wrong! {:?} {:?} Angle({:?}) {:?} {:?}", angle1, angle2, angle2.to_f64(), x, y);
            }
        }
    }

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let _ = f32::atan2(y as f32, x as f32);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard atan2 time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let _ = fb.atan2(Fast(y), Fast(x));
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Fast atan2 time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let xf = x as f32;
            let yf = y as f32;
            let _ = f32::sqrt(xf * xf + yf * yf);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard sqrt time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let xf = x as f32;
            let yf = y as f32;
            let _ = fb.dist((Fast(xf as i64),Fast(yf as i64)));
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Fast sqrt time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let angl = f64::atan2(y as f64,x as f64);
            let _ = f64::cos(angl) * 1024.0;
            let _ = f64::sin(angl) * 1024.0;
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard offset time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..1024 {
        for y in 0..1024 {
            let _ = fb.offset(fb.atan2(Fast(y as i64), Fast(x as i64)), Fast(1024i64));
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Fast offset time: {}ms", total as f32 / mili);
}