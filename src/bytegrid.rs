type Point = (isize,isize);

pub struct ByteGrid {
    pub w:      isize,
    pub h:      isize,
    vec:        Vec<u8>,
}

impl ByteGrid {
    pub fn new(w: isize, h: isize) -> ByteGrid {
        let mut vec = Vec::with_capacity((w * h) as usize);

        for _ in 0..(w * h) {
            vec.push(0);
        }
        ByteGrid {w: w, h: h, vec: vec}
    }

    pub fn is_open(&self, (x,y): Point) -> bool {
        x >= 0     &&
        y >= 0     &&
        x < self.w &&
        y < self.h &&
        self.vec[(y * self.w + x) as usize] == 0
    }

    pub fn set_point(&mut self, v: u8, (x,y): Point) {
        self.vec[(y * self.w + x) as usize] = v
    }

    pub fn get_point(&self, (x,y): Point) -> u8 {
        self.vec[(y * self.w + x) as usize]
    }

    pub fn correct_move(&self, a: (f32,f32), b: (f32,f32)) -> (f32,f32) {
        let (x0,y0) = a;
        let (x1,y1) = b;

        let (x,y) = self.last_open( (x0.floor() as isize, y0.floor() as isize)
                                  , (x1.floor() as isize, y1.floor() as isize));
        let (xf,yf) = (x as f32, y as f32);
        let min_x = xf + 0.0001;
        let max_x = xf + 0.9999;
        let min_y = yf + 0.0001;
        let max_y = yf + 0.9999;

        let mut new_x = x1;
        let mut new_y = y1;

        if x1 > max_x {
            new_x = max_x;
        }
        else if x1 < min_x {
            new_x = min_x;
        }

        if y1 > max_y {
            new_y = max_y;
        }
        else if y1 < min_y {
            new_y = min_y;
        }

        (new_x, new_y)
        /*
        match self.last_open((x0.floor() as isize, y0.floor() as isize), (x1.floor() as isize, y1.floor() as isize)) {
            None => b,
            Some(xy) => {
                let (x,y) = xy;
                let min_x_bound = |pxy: Point| {
                    let (px,_) = pxy;
                    if !self.is_open(pxy) {
                        px as f32 - 0.001
                    }
                    else {
                        x1
                    }
                };
                let max_x_bound = |pxy: Point| {
                    let (px,_) = pxy;
                    if !self.is_open(pxy) {
                        px as f32 + 1.001
                    }
                    else {
                        x1
                    }
                };
                let min_y_bound = |pxy: Point| {
                    let (_,py) = pxy;
                    if !self.is_open(pxy) {
                        py as f32 - 0.001
                    }
                    else {
                        y1
                    }
                };
                let max_y_bound = |pxy: Point| {
                    let (_,py) = pxy;
                    if !self.is_open(pxy) {
                        py as f32 + 1.001
                    }
                    else {
                        y1
                    }
                };
                let new_xy = match direction_from_points((x0.floor() as isize, y0.floor() as isize), xy) {
                    None => (x1,y1),
                    Some(dir) => match dir {
                        NORTH     => (x1                                  , y as f32 - 0.001                   ),
                        SOUTH     => (x1                                  , y as f32 + 1.001                   ),
                        EAST      => (x as f32 - 0.001                    , y1                                 ),
                        WEST      => (x as f32 + 1.001                    , y1                                 ),
                        NORTHEAST => (min_x_bound(translate(1, SOUTH, xy)), min_y_bound(translate(1, WEST, xy))),
                        NORTHWEST => (max_x_bound(translate(1, SOUTH, xy)), min_y_bound(translate(1, EAST, xy))),
                        SOUTHEAST => (min_x_bound(translate(1, NORTH, xy)), max_y_bound(translate(1, WEST, xy))),
                        SOUTHWEST => (max_x_bound(translate(1, NORTH, xy)), max_y_bound(translate(1, EAST, xy))),
                        _         => panic!("correct_move failed with a bad direction."),
                    }
                };

                let (nx,ny) = xy;
                let dif_x = nx as f32 - x0;
                let dif_y = ny as f32 - y0;
                let dist = dif_x * dif_x + dif_y * dif_y;
                if dist > 2.0 {
                    println!("{:?}: {} -> {:?}: {} to {:?}: {}", a, self.is_open((x0.floor() as isize, y0.floor() as isize)), b, self.is_open((x1.floor() as isize, y1.floor() as isize)), xy, self.is_open((nx,ny)));
                }
                new_xy
            }
        }
        */
    }

    pub fn last_open(&self, (x0,y0): Point, (x1,y1): Point) -> Point {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let x_inc = if x1 > x0 {1} else {-1};
        let y_inc = if y1 > y0 {1} else {-1};

        let mut x = x0;
        let mut y = y0;
        let mut err = dx - dy;
        let mut prev_x = x;
        let mut prev_y = y;

        loop {
            if !self.is_open((x,y)) {
                return (prev_x, prev_y);
            }
            if x == x1 && y == y1 {
                return (x, y);
            }
            if err == 0 {
                if !self.is_open((x + x_inc, y)) && !self.is_open((x, y + y_inc)) {
                    return (x, y);
                }
                prev_x = x;
                prev_y = y;
                x += x_inc;
                y += y_inc;
                err = err - dy + dx;
            }
            else if err > 0 {
                prev_x = x;
                x += x_inc;
                err -= dy;
            }
            else {
                prev_y = y;
                y += y_inc;
                err += dx;
            }
        }
    }
    /*
    pub fn first_closed(&self, (x0,y0): (f32,f32), (x1,y1): (f32,f32)) -> Option<Point> {
        let ix0 = x0.floor() as isize;
        let iy0 = y0.floor() as isize;
        let ix1 = x1.floor() as isize;
        let iy1 = y1.floor() as isize;
        let start = (ix0,iy0);
        let end = (ix1,iy1);

        match direction_from_points(start, end) {
            None => None,
            Some(dir) => {
                match dir {
                    NORTH | SOUTH | EAST | WEST => {
                        let mut xy = translate(1, dir, start);

                        loop {
                            if !self.is_open(xy) {
                                return Some(xy);
                            }
                            if xy == end {
                                return None;
                            }
                            xy = translate(1, dir, xy);
                        }
                    }
                    _ => {
                        let step_x = if x1 > x0 { 1.0 } else { -1.0 };
                        let step_y = if y1 > y0 { 1.0 } else { -1.0 };
                        let dif_x = x1 - x0;
                        let dif_y = y1 - y0;
                        let x_delta = 1.0 / dif_x;
                        let y_delta = 1.0 / dif_y;
                        let x_max = x_delta * (1.0 - x0);
                        let y_max = y_delta * (1.0 - y0);
                        let mut x = x0;
                        let mut y = y0;
                        let mut xm = x_max;
                        let mut ym = y_max;

                        loop {
                            let fx = x.floor() as isize;
                            let fy = y.floor() as isize;
                            let fp = (fx,fy);

                            if !self.is_open(fp) {
                                return Some(fp);
                            }
                            else if fp == end {
                                return None;
                            }
                            else if xm == ym {
                                let xs = x + step_x;
                                let ys = y + step_y;
                                let fxs = xs.floor() as isize;
                                let fys = ys.floor() as isize;
                                if !self.is_open((fxs, fy)) && !self.is_open((fx, fys)) {
                                    return Some((fxs, fys));
                                }
                                else {
                                    x = xs;
                                    y = ys;
                                    xm += x_delta;
                                    ym += y_delta;
                                }
                            }
                            else if xm < ym {
                                x += step_x;
                                xm += x_delta;
                            }
                            else {
                                y += step_y;
                                ym += y_delta;
                            }
                        }
                    }
                }
            }
        }
    }
    */
}


fn direction_from_points((x0,y0): Point, (x1,y1): Point) -> Option<Direction> {
    if x1 > x0 {
        if y1 > y0 {
            Some(NORTHEAST)
        }
        else if y1 == y0 {
            Some(EAST)
        }
        else {
            Some(SOUTHEAST)
        }
    }
    else if x1 == x0 {
        if y1 > y0 {
            Some(NORTH)
        }
        else if y1 == y0 {
            None
        }
        else {
            Some(SOUTH)
        }
    }
    else {
        if y1 > y0 {
            Some(NORTHWEST)
        }
        else if y1 == y0 {
            Some(WEST)
        }
        else {
            Some(SOUTHWEST)
        }
    }
}

#[derive(Clone,Copy)]
struct Direction(isize);
const NORTH:        Direction = Direction(0);
const SOUTH:        Direction = Direction(4);
const EAST:         Direction = Direction(2);
const WEST:         Direction = Direction(6);
const NORTHEAST:    Direction = Direction(1);
const SOUTHEAST:    Direction = Direction(3);
const SOUTHWEST:    Direction = Direction(5);
const NORTHWEST:    Direction = Direction(7);

fn translate(n: isize, Direction(dir): Direction, (x,y): Point) -> Point {
    match dir {
        0 => return (x, y + n),
        1 => return (x + n, y + n),
        2 => return (x + n, y),
        3 => return (x + n, y - n),
        4 => return (x, y - n),
        5 => return (x - n, y - n),
        6 => return (x - n, y),
        7 => return (x - n, y + n),
        _ => panic!("translate was given a bad Direction.")
    }
}

pub fn test() {
    let w = 3;
    let h = 3;
    let bg = ByteGrid::new(w,h);
    for y0 in 0..h {
        for x0 in 0..w {
            for y1 in 0..h {
                for x1 in 0..w {
                    let _ = bg.last_open((x0,y0), (x1,y1));
                    println!("");
                }
            }
        }
    }
}