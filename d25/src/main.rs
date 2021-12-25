use std::collections::HashMap;
use std::fmt;
use std::io;
use std::io::{BufRead};
use std::ops::{Add, Deref, Rem};

type ParseError = &'static str;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Cucumber {
    Easterly,
    Southerly,
}
type Int = u32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Pos(Int, Int);

impl From<[Int; 2]> for Pos {
    fn from([x, y]: [Int; 2]) -> Pos {
        Pos(x, y)
    }
}
impl From<[usize; 2]> for Pos {
    fn from([x, y]: [usize; 2]) -> Pos {
        Pos(x as Int, y as Int)
    }
}
impl Default for Pos {
    fn default() -> Pos { Pos(0,0) }
}

impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Self::Output {
        Pos(self.0 + other.0, self.1 + other.1)
    }
}
impl<B> Rem<&B> for Pos where B: PosBound {
    type Output = Pos;
    fn rem(self, bound: &B) -> Self::Output {
        Pos(self.0 % bound.width(), self.1 % bound.height())
    }
}

trait PosBound {
    fn width(&self) -> Int;
    fn height(&self) -> Int;
}

#[derive(Clone)]
struct Map {
    locations: HashMap<Pos, Cucumber>,
    width: Int,
    height: Int,
}

impl Map {
    fn try_from_lines<'a, I, S>(lines: I) -> Result<Map, ParseError>
    where
        I: Iterator<Item = S>,
        S: Deref<Target = str>,
    {
        let mut width = 0;
        let mut height = 0;
        let mut locations = HashMap::with_capacity(100);

        for (r, l) in lines.enumerate() {
            width = l.len() as Int;
            height += 1;
            for (c, &ch) in l.as_bytes().iter().enumerate() {
                match Cucumber::try_from(ch) {
                    Ok(cuc) => { locations.insert([c,r].into(), cuc); },
                    Err(_) => (),
                }
            }
        }
        Ok(Map { locations, width, height })
    }

    fn step(&mut self) -> usize {
        self.step_herd(Cucumber::Easterly) + self.step_herd(Cucumber::Southerly)
    }

    fn step_herd(&mut self, herd: Cucumber) -> usize {
        let locations = &self.locations;
        let mut movements = Vec::with_capacity(locations.len() / 2);

        for (pos, c) in locations.iter().filter(|(_, &c)| c == herd) {
            let next = (*pos + match c {
                Cucumber::Easterly => Pos(1, 0),
                Cucumber::Southerly => Pos(0, 1),
            }) % self;

            if !locations.contains_key(&next) {
                movements.push((*pos, next, *c));
            }
        }

        let locations = &mut self.locations;
        for (from, to,  c) in &movements {
            locations.remove(from);
            locations.insert(*to, *c);
        }

        movements.len()
    }
}
impl PosBound for Map {
    fn width(&self) -> Int { self.width }
    fn height(&self) -> Int { self.height }
}

impl TryFrom<u8> for Cucumber {
    type Error = ParseError;
    fn try_from(c: u8) -> Result<Cucumber, ParseError> {
        match c {
            b'>' => Ok(Cucumber::Easterly),
            b'v' => Ok(Cucumber::Southerly),
            _ => Err("Not a cucumber"),
        }
    }
}
impl fmt::Display for Cucumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Cucumber::Easterly => '>',
            Cucumber::Southerly => 'v',
        })
    }
}
impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let map = &self.locations;
        for r in 0..self.height {
            for c in 0..self.width {
                match map.get(&[c, r].into()) {
                    Some(c) => write!(f, "{}", c)?,
                    None => write!(f, ".")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn read_input() -> Map {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap());
    
    Map::try_from_lines(lines).unwrap()
}

fn steps_to_stop(map: &Map) -> usize{
    let map = &mut map.clone();
    let mut steps = 0;
    loop {
        steps += 1;
        if map.step() == 0 {
            return steps;
        }
    }
}

fn main() {
    let mut map = read_input();

    println!("{}x{}", &map.width(), &map.height());
    println!("{:?}", &map);

    let steps = steps_to_stop(&map);
    println!("{} steps to stop", steps);

    /*
    for step in 1..=5 {
        map.step();
        println!("After step {}", step);
        println!("{:?}", &map);
    }
    */
}
