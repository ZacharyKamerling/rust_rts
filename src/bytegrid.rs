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

        match self.first_closed(a,b) {
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
                match direction_from_points((x0.floor() as isize, y0.floor() as isize), xy) {
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
                }
            }
        }
    }

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
}

/*
isPathOpen :: IsOpen -> Point -> Point -> Bool
isPathOpen isOpen (x0,y0) (x1,y1) = rat (1 + dx + dy) x0 y0 err
    where
    dx = abs (x1 - x0)
    dy = abs (y1 - y0)
    err = dx - dy
    x_inc = if x1 > x0 then 1 else -1
    y_inc = if y1 > y0 then 1 else -1
    rat 0 _ _ _ = True
    rat c x y e =
        isOpen (x, y) &&
          ((x == x1 && y == y1) ||
             (if e == 0 then
                not (eitherOpen (x + x_inc, y) (x, y + y_inc)) ||
                  rat (c - 1) (x + x_inc) (y + y_inc) (e - dy + dx)
                else
                if e > 0 then rat (c - 1) (x + x_inc) y (e - dy) else
                              rat (c - 1) x (y + y_inc) (e + dx)))
    eitherOpen axy bxy = isOpen axy || isOpen bxy
*/

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