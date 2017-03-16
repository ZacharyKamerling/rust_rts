extern crate time;

use self::time::{PreciseTime};
use std::f64::consts::{PI};
use std::f64;

#[derive(Clone,Copy,Debug)]
pub struct TinyAngle {
    data: u16,
}

impl TinyAngle {
    pub fn to_f64(&self) -> f64 {
        self.data as f64 * PI * 2.0 / 65536.0
    }
}

#[derive(Clone,Copy,Debug)]
pub struct TinyFloat {
    data: i32,
}

pub struct TinyBase {
    cos_sin: Vec<(u8,u8)>,
    atan2: Vec<u16>,
}

impl TinyBase {
    pub fn new() -> TinyBase {
        let mut a = Vec::with_capacity(16384);
        let mut b = Vec::with_capacity(1048576);

        let ix = (PI / 2.0) / 16384.0;
        for i in 0..16384 {
            let f = i as f64 * ix;
            let xo = f64::cos(f);
            let yo = f64::sin(f);

            a.push(((xo * 256.0) as u8, (yo * 256.0) as u8));
        }

        for y in 0..1024 {
            for x in 0..1024 {
                let angl = (f64::atan2(y as f64, x as f64) / (PI / 2.0) * 16384.0) as u16;
                b.push(angl);
            }
        }

        TinyBase {
            cos_sin: a,
            atan2: b,
        }
    }

    pub fn atan2(&self, ffy: TinyFloat, ffx: TinyFloat) -> TinyAngle {
        let x = ffx.data;
        let y = ffy.data;

        if x >= 0 {
            if y >= 0 {
                // NE
                TinyAngle { data: self.qrtr_atan2(y, x) }
            }
            else {
                // SE
                TinyAngle { data: 65535 - self.qrtr_atan2(y * -1, x) }
            }
        }
        else {
            // NW
            if y >= 0 {
                TinyAngle { data: 32768 - self.qrtr_atan2(y, x * -1) }
            }
            else {
            //SW
                TinyAngle { data: 32768 + self.qrtr_atan2(y * -1, x * -1) }
            }
        }
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

    pub fn offset(&self, a: TinyAngle, (ffx,ffy): (TinyFloat,TinyFloat), offset: TinyFloat) -> (TinyFloat,TinyFloat) {
        let (xo,yo) = self.cos_sin[a.data as usize];
        let (x,y) = (ffx.data as i64 * xo as i64 / 255, ffy.data as i64 * yo as i64 / 255);

        (TinyFloat {data : x as i32}, TinyFloat {data: y as i32})
    }
}

pub fn bench() {
    let mili = 1000000.0;
    let tb = TinyBase::new();

    let start = PreciseTime::now();
    for x in 0..10240 {
        for y in 0..10240 {
            let angl = f32::atan2(y as f32, x as f32);
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("Standard time: {}ms", total as f32 / mili);

    let start = PreciseTime::now();
    for x in 0..10240 {
        for y in 0..10240 {
            let angl = tb.atan2(TinyFloat {data: y as i32}, TinyFloat {data: x as i32});
        }
    }
    let end = PreciseTime::now();

    let total = start.to(end).num_nanoseconds().unwrap();
    println!("TinyFloat time: {}ms", total as f32 / mili);
}