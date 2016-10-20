use std::collections::BinaryHeap;
use std::cmp::Ordering;
use pathing::direction as dir;

pub type Point = (isize,isize);

#[derive(Clone)]
pub struct PathGrid {
    w: isize,
    h: isize,
    states: Vec<u8>,
    jumps: Vec<Jumps>,
    // Avoid allocations by using these pre-allocated collections
    open: BinaryHeap<Node>,
    expand: Vec<(Point, Direction)>,
    closed: HashSet<Point>,
    came_from: HashMap<Point,Point>,
}

#[derive(Clone,Debug,Eq,PartialEq)]
struct Node(DistSearched,Point,Direction);

struct Node {
    dist_searched: u16,
    xy: (isize,isize),
    direction: Direction,
}

#[derive(Clone,Debug)]
struct Jumps {
    nj: u16,
    ej: u16,
    sj: u16,
    wj: u16,
}