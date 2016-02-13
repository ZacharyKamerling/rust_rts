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

    #[inline]
    pub fn is_open(&self, (x,y): Point) -> bool {
        (x >= 0)     &
        (y >= 0)     &
        (x < self.w) &
        (y < self.h) &&
        self.vec[(y * self.w + x) as usize] == 0
    }

    pub fn set_point(&mut self, v: u8, (x,y): Point) {
        self.vec[(y * self.w + x) as usize] = v
    }

    pub fn get_point(&self, (x,y): Point) -> u8 {
        self.vec[(y * self.w + x) as usize]
    }

    pub fn correct_move(&self, a: (f32,f32), b: (f32,f32)) -> (f32,f32,bool,bool) {
        let (x0,y0) = a;
        let (x1,y1) = b;

        let (x,y) = self.last_open( (x0 as isize, y0 as isize)
                                  , (x1 as isize, y1 as isize));
        let (xf,yf) = (x as f32, y as f32);
        let min_x = xf + 0.01;
        let max_x = xf + 0.99;
        let min_y = yf + 0.01;
        let max_y = yf + 0.99;

        let mut new_x = x1;
        let mut new_y = y1;
        let mut x_changed = false;
        let mut y_changed = false;

        if !self.is_open((x + 1, y)) && x1 > max_x {
            new_x = max_x;
            x_changed = true;
        }
        else if !self.is_open((x - 1, y)) && x1 < min_x {
            new_x = min_x;
            x_changed = true;
        }

        if !self.is_open((x, y + 1)) && y1 > max_y {
            new_y = max_y;
            y_changed = true;
        }
        else if !self.is_open((x, y - 1)) && y1 < min_y {
            new_y = min_y;
            y_changed = true;
        }

        (new_x, new_y, x_changed, y_changed)
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