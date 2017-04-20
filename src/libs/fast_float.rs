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
        else {
            if a > b {
                if a - b > u16::max_value() / 2 {
                    Angle(a + turn)
                }
                else {
                    Angle(a - turn)
                }
            }
            else {
                if b - a > u16::max_value() / 2 {
                    Angle(a - turn)
                }
                else {
                    Angle(a + turn)
                }
            }
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
    fraction: i64,
    coeff: i64,
    cos_sin: Vec<(i32,i32)>,
    atan2: Vec<u16>,
}

impl FastBase {

    pub fn new() -> FastBase {
        let fraction = 2^50;
        let coeff = 2^13;
        let mut cos_sin = Vec::with_capacity(2^16);
        let mut atan2 = Vec::with_capacity(2^13 * 2^13);

        let ix = (2^16) as f64;
        for i in 0..2^16 {
            let f = i as f64 / u16::max_value() as f64 * PI * 2.0;
            let xo = f64::cos(f);
            let yo = f64::sin(f);

            cos_sin.push(((xo * ix) as i32, (yo * ix) as i32));
        }

        let qrtr = ix / 4.0;
        for y in 0..coeff {
            for x in 0..coeff {
                let angle = (f64::atan2(y as f64, x as f64) / (PI / 2.0) * qrtr) as u16;
                atan2.push(angle);
            }
        }

        FastBase {
            fraction: fraction,
            coeff: coeff,
            atan2: atan2,
            cos_sin: cos_sin,
        }
    }

    pub fn coord(&self, f: f64) -> Fast {
        Fast((f * (self.fraction / self.coeff) as f64) as i64)
    }

    pub fn atan2(&self, Fast(y): Fast, Fast(x): Fast) -> Angle {

        Angle(
            if x >= 0 {
                if y >= 0 {
                    // NE
                    self.qrtr_atan2(y, x)
                }
                else {
                    // SE
                    u16::max_value() - self.qrtr_atan2(y * -1, x)
                }
            }
            else {
                // NW
                if y >= 0 {
                    u16::max_value() / 2 - self.qrtr_atan2(y, x * -1)
                }
                else {
                //SW
                    u16::max_value() / 2 + self.qrtr_atan2(y * -1, x * -1)
                }
            }
        )
    }

    fn qrtr_atan2(&self, mut y: i64, mut x: i64) -> u16 {
        x = x.abs();
        y = y.abs();
        let d = {
            if x > y {
                x / self.coeff + 1
            }
            else {
                y / self.coeff + 1
            }
        };

        x /= d;
        y /= d;

        self.atan2[(y * self.coeff + x) as usize]
    }

    pub fn dist(&self, (Fast(x),Fast(y)): (Fast,Fast)) -> Fast {
        /*
        x & y are 50 bits. We need to reduce them to 30 bits before squaring & adding them.
        */
        let denom = 2^20;
        let x = x / denom;
        let y = y / denom;

        Fast(f64::sqrt((x * x + y * y) as f64) as i64 * denom)
    }

    pub fn in_range(&self, (Fast(x),Fast(y)): (Fast,Fast), Fast(r): Fast) -> bool {
        /*
        x & y are 50 bits. We need to reduce them to 30 bits before squaring & adding them.
        */
        let denom = 2^20;
        let x = x / denom;
        let y = y / denom;
        let r = r / denom;

        x * x + y * y < r * r
    }
}