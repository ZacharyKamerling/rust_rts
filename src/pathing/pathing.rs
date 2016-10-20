use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct JumpGrid {
    w: isize,
    h: isize,
    open_vec: [u8; 1024 * 1024],
    jump_vec: [Jumps; 1024 * 1024],
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