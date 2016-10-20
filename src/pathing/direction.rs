#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct Direction(isize);

#[derive(Clone,Copy,Debug)]
pub struct Degree(isize);

pub const DEG_45:       Degree = Degree(1);
pub const DEG_90:       Degree = Degree(2);
pub const DEG_135:      Degree = Degree(3);
pub const DEG_180:      Degree = Degree(4);
pub const NORTH:        Direction = Direction(0);
pub const EAST:         Direction = Direction(2);
pub const SOUTH:        Direction = Direction(4);
pub const WEST:         Direction = Direction(6);
pub const NORTHEAST:    Direction = Direction(1);
pub const SOUTHEAST:    Direction = Direction(3);
pub const SOUTHWEST:    Direction = Direction(5);
pub const NORTHWEST:    Direction = Direction(7);

pub fn rotate_c(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d + rot;
    if d2 >= 8 {
        return Direction(d2 - 8);
    }
    else {
        return Direction(d2);
    }
}

pub fn rotate_cc(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d - rot;
    if d2 < 0 {
        return Direction(8 + d2);
    }
    else {
        return Direction(d2);
    }
}

pub fn translate(n: isize, Direction(dir): Direction, (x,y): (isize,isize)) -> (isize,isize) {
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