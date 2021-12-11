use std::io;
use std::io::BufRead;
use std::cmp;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct Pos {
    r: usize,
    c: usize,
}
impl Pos {
    const fn new(r: usize, c: usize) -> Pos {
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

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct Octopus(u8, bool);

impl Octopus {
    pub fn with_energy(e: u8) -> Octopus {
        Octopus(e, false)
    }

    pub const fn is_stepping(&self) -> bool { self.1 }
    pub const fn will_flash(&self) -> bool { self.0 >= 10 }
    pub const fn flashed(&self) -> bool { self.0 == 0 }

    pub fn inc_energy(&mut self) -> bool {
        self.1 = true;
        self.0 = cmp::min(self.0 + 1, 11);
        self.0 == 10
    }

    pub fn finish_step(&mut self) -> bool {
        self.1 = false;
        if self.0 > 9 {
            self.0 = 0;
            true
        } else {
            false
        }
    }
}

impl fmt::Debug for Octopus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for Octopus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct OctoMap(Vec<Octopus>, usize);

impl OctoMap {
    pub fn from_str(lines: impl Iterator<Item = String>) -> OctoMap {
        let mut map = Vec::with_capacity(100);
        let mut width = 0;
        for l in lines {
            let s = l.trim_end();
            map.extend(s.chars().map(|c| Octopus::with_energy(c.to_digit(10).unwrap() as u8)));
            width = s.len();
        }
        OctoMap(map, width)
    }

    pub fn height(&self) -> usize {
        self.0.len() / self.1
    }

    pub fn width(&self) -> usize {
        self.1
    }

    pub fn step(&mut self) -> u32 {
        let mut will_flash = vec!();

        for p in self.positions() {
            let o = &mut self[p];
            if o.inc_energy() {
                will_flash.push(p);
                //println!("flash! {}", p);
            }
        }

        loop {
            let center = match will_flash.pop() {
                Some(p) => p,
                None => break,
            };

            for dr in -1..=1 {
                for dc in -1..=1 {
                    let adj_pos = match self.adjacent(center, dr, dc) {
                        Some(p) => p,
                        None => continue,
                    };

                    let adj = &mut self[adj_pos];
                    if adj.inc_energy() {
                        will_flash.push(adj_pos);
                        //println!("induced flash! {}", adj_pos);
                    }
                }
            }
        }

        let mut flashed = 0;
        for p in self.positions() {
            let o = &mut self[p];
            if o.finish_step() {
                flashed += 1;
            }
        }

        flashed
    }

    fn positions(&self) -> impl Iterator<Item = Pos> {
        GridTraverse::with_size(self.1, self.0.len() / self.1)
    }

    fn adjacent(&self, pos: Pos, r_offset: isize, c_offset: isize) -> Option<Pos> {
        let r = match r_offset {
            d if d > 0 => pos.r + r_offset as usize,
            d if d < 0 => match pos.r.overflowing_sub(-r_offset as usize) {
                (r, false) => r,
                (_, true) => return None,
            },
            _ => pos.r,
        };

        let c = match c_offset {
            d if d > 0 => pos.c + c_offset as usize,
            d if d < 0 => match pos.c.overflowing_sub(-c_offset as usize) {
                (c, false) => c,
                (_, true) => return None,
            },
            _ => pos.c,
        };

        if r < self.height() && c < self.width() {
            Some(Pos::new(r, c))
        } else {
            None
        }
    }
}

impl std::ops::Index<Pos> for OctoMap {
    type Output = Octopus;
    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r * self.1 + index.c]
    }
}
impl std::ops::IndexMut<Pos> for OctoMap {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r * self.1 + index.c]
    }
}

impl fmt::Display for OctoMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(
            for r in 0..(self.0.len() / self.1) {
                for c in 0..self.1 {
                    write!(f, "{}", self[Pos::new(r, c)])?;
                }
                writeln!(f)?;
            }
        )
    }
}

pub struct GridTraverse {
    i: usize,
    width: usize,
    height: usize,
}

impl GridTraverse {
    fn with_size(width: usize, height: usize) -> GridTraverse {
        GridTraverse { i: 0, width, height }
    }
}

impl Iterator for GridTraverse {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.width * self.height {
            return None;
        }

        let pos = Pos::new(self.i / self.width, self.i % self.width);
        self.i += 1;
        Some(pos)
    }
}

pub struct OctoMapTraverse<'a> {
    map: &'a OctoMap,
    i: GridTraverse,
}

impl<'a> Iterator for OctoMapTraverse<'a> {
    type Item = (Pos, &'a Octopus);

    fn next(&mut self) -> Option<Self::Item> {
        match self.i.next() {
            Some(p) => Some((p, &self.map[p])),
            None => None,
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut map = OctoMap::from_str(stdin.lock().lines().map(|l| l.unwrap()));

    println!("{}", &map);

    let mut flashes = 0;
    let mut first_sync = None;
    for step in 1..=1000 {
        let f = map.step();
        flashes += f;
        println!("step {}", step);
        println!("{}", &map);

        if f == 100 && first_sync.is_none() {
            first_sync = Some(step);
        }
    }
    println!("{} flashes", flashes);
    println!("first sync: step {}", first_sync.unwrap_or(-1));
}
