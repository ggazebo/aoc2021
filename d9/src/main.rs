use std::fmt;
use std::io::BufRead;
use std::iter::Iterator;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Height(u8);

impl Height {
    pub fn from_char(c: char) -> Option<Height> {
        Some(Height(c.to_digit(10)? as u8))
    }

    pub fn risk_level(&self) -> u32 {
        self.0 as u32 + 1
    }
}

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(write!(f, "{}", self.0)?)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct Pos {
    r: usize,
    c: usize,
}
impl Pos {
    fn new(r: usize, c: usize) -> Pos {
        Pos{r, c}
    }
}
impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.r, self.c)
    }
}

type HeightInfo<'a> = (Pos, &'a Height);

pub struct HeightMap {
    map: Vec<Height>,
    width: usize,
    height: usize,
}

impl HeightMap {
    pub fn from_str(lines: impl Iterator<Item = String>) -> HeightMap {
        let mut map = Vec::with_capacity(256);
        let mut width = 0;
        let mut height = 0;
        for (h, l) in lines.enumerate() {
            let s = l.trim_end();
            map.extend(s.chars().map(|c| Height::from_char(c).unwrap()));
            width = s.len();
            height = h;
        }
        height += 1;
        HeightMap { map, width, height }
    }

    pub fn iter_with_pos<'a>(&'a self) -> HeightMapValues<'a> {
        HeightMapValues { map: &self, p: Default::default() }
    }

    pub fn adjacents<'a>(&'a self, p: Pos) -> impl Iterator<Item = HeightInfo<'a>> {
        let pos_it = AdjacentPos{ origin: p, dir: Adjacency::None, w: self.width, h: self.height };
        pos_it.map(|p| (p, &self[p]))
    }
}

impl std::ops::Index<Pos> for HeightMap {
    type Output = Height;

    fn index(&self, p: Pos) -> &Self::Output {
        &self.map[p.r * self.width + p.c]
    }
}

pub struct HeightMapValues<'a> {
    map: &'a HeightMap,
    p: Pos,
}

impl<'a> Iterator for HeightMapValues<'a> {
    type Item = HeightInfo<'a>;
    //type Item = (Pos, &'a Height);

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.p;

        self.p.c = (p.c + 1) % self.map.width;
        if self.p.c == 0 {
            self.p.r += 1;
            if self.p.r > self.map.height {
                return None;
            }
        }

        Some((p, &self.map[p]))
    }
}

enum Adjacency {
    None,
    Up,
    Right,
    Down,
    Left,
}

struct AdjacentPos {
    origin: Pos,
    dir: Adjacency,
    h: usize,
    w: usize,
}

impl Iterator for AdjacentPos {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.dir = match self.dir {
                Adjacency::None => Adjacency::Up,
                Adjacency::Up => Adjacency::Right,
                Adjacency::Right => Adjacency::Down,
                Adjacency::Down => Adjacency::Left,
                Adjacency::Left => return None,
            };

            match self.dir {
                Adjacency::Up if self.origin.r > 0 => return Some(Pos::new(self.origin.r - 1, self.origin.c)),
                Adjacency::Right if self.origin.c < self.w-1 => return Some(Pos::new(self.origin.r, self.origin.c + 1)),
                Adjacency::Down if self.origin.r < self.h-1 => return Some(Pos::new(self.origin.r + 1, self.origin.c)),
                Adjacency::Left if self.origin.c > 0 => return Some(Pos::new(self.origin.r, self.origin.c - 1)),
                _ => continue,
            };
        }
    }
}

fn part1(map: &HeightMap) {
    let h = map.height;
    let w = map.width;
    let mut it = map.iter_with_pos();
    let mut lows = Vec::with_capacity(64);
    for _ in 0..h {
        for _ in 0..w {
            //print!("{}", map.heights[(r * w + c) as usize]);
            let (p, h) = it.next().unwrap();
            print!("{}", h);

            if map.adjacents(p).all(|(_, ah)| ah > h) {
                lows.push((p, h));
            }
        }
        println!("");
    }

    println!("lows:");
    for (p, h) in &lows {
        println!("{}:{}", p, h);
    }

    let risk: u32 = (&lows).iter().map(|(_, h)| h.risk_level()).sum();
    println!("risk: {}", risk);
}

fn main() {
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap());
    let map = HeightMap::from_str(lines);

    println!("map dim: {}x{}", map.width, map.height);

    part1(&map);
}
