use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops::{Deref};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pos(usize, usize);
impl Pos {
    fn x(&self) -> usize { self.0 }
    fn y(&self) -> usize { self.1 }
}

impl From<[usize; 2]> for Pos {
    fn from(v: [usize; 2]) -> Pos {
        Pos(v[0], v[1])
    }
}
impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

pub trait ChitonCave {
    fn dim(&self) -> usize;

    fn entrance(&self) -> Pos {
        [0, 0].into()
    }

    fn exit(&self) -> Pos {
        let d = self.dim();
        [d-1, d-1].into()
    }

    fn risk(&self, p: Pos) -> Risk;
}

pub trait CaveMap {
    fn best_path(&self) -> Option<(Vec<Pos>, Risk)>;
    fn neighbours(&self, p: Pos) -> AdjacentPositions;
}

impl<C: ChitonCave> CaveMap for C {
    fn best_path(&self) -> Option<(Vec<Pos>, Risk)> {
        use pathfinding::directed::astar::astar;

        astar(
            &self.entrance(),
            |&o| self.neighbours(o).map(|p| (p, self.risk(p))),
            |p| (self.dim() * 2 - p.x() - p.y()) as Risk,
            |p| *p == self.exit())
    }

    fn neighbours(&self, p: Pos) -> AdjacentPositions {
        AdjacentPositions::from_pos(p, self.dim())
    }
}

pub struct AdjacentPositions {
    origin: Pos,
    dim: usize,
    n: u8,
}
impl AdjacentPositions {
    pub fn from_pos(p: Pos, dim: usize) -> AdjacentPositions {
        AdjacentPositions { origin: p, dim, n: 0 }
    }
}
impl Iterator for AdjacentPositions {
    type Item = Pos;
    fn next(&mut self) -> Option<Pos> {
        let x = self.origin.x();
        let y = self.origin.y();
        loop {
            self.n += 1;
            match self.n {
                1 if y > 0 => return Some([x, y - 1].into()),
                2 if x + 1 < self.dim => return Some([x + 1, y].into()),
                3 if y + 1 < self.dim => return Some([x, y + 1].into()),
                4 if x > 0 => return Some([x - 1, y].into()),
                n if n > 4 => return None,
                _ => (),
            }
        }
    }
}

pub struct Cave {
    dim: usize,
    risks: Vec<u8>,
}
impl ChitonCave for Cave {
    fn dim(&self) -> usize { self.dim }

    fn risk(&self, p: Pos) -> Risk {
        self.risks[p.y() * self.dim() + p.x()] as Risk
    }
}
impl Cave {
    pub fn from_reader<I, L>(lines: &mut I) -> Cave
    where I: Iterator<Item = L>, L: Deref<Target = str> {
        let mut risks = Vec::with_capacity(100);
        let mut dim = 0;

        for l in lines {
            let bytes = l.as_bytes();
            dim = bytes.len();
            risks.extend(bytes.iter().map(|b| b - b'0'));
        }

        Cave { dim, risks }
    }
}

pub struct ExtendedCave<'a> {
    cave: &'a Cave,
    repeat: usize,
}
impl<'a> ExtendedCave<'a> {
    pub fn from_cave(cave: &'a Cave, repeat: usize) -> ExtendedCave {
        ExtendedCave { cave, repeat }
    }
}
impl<'cave> ChitonCave for ExtendedCave<'cave> {
    fn dim(&self) -> usize { self.cave.dim() * self.repeat }

    fn risk(&self, p: Pos) -> Risk {
        let d = self.cave.dim();
        let dr = p.x() / d + p.y() / d;
        let r = self.cave.risk([p.x() % d, p.y() % d].into());

        (r - 1 + dr as Risk) % 9 + 1
    }
}

pub type Risk = u32;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().map(|l| l.unwrap());
    let cave = Cave::from_reader(&mut lines);

    println!("dimensions: {0}x{0}", cave.dim());

    match cave.best_path() {
        Some((_p, c)) => {
            println!("shortest path: {}", c);
            //println!("{:?}", p);
        },
        None => println!("NO PATH"),
    }

    println!();

    let cave = ExtendedCave::from_cave(&cave, 5);
    println!("dimensions: {0}x{0}", cave.dim());

    match cave.best_path() {
        Some((_p, c)) => {
            println!("shortest path: {}", c);
        },
        None => println!("NO PATH"),
    }
}
