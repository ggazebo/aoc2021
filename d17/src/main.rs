use std::cmp;
use std::cmp::{Ord};
use std::fmt;
use std::io;
use std::io::{BufRead};
use std::iter;
use std::ops::{RangeInclusive, Add};
use std::str::{FromStr};

pub type Int = i32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(Int, Int);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Velocity(Int, Int);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    p: Pos,
    v: Velocity,
}

impl Probe {
    pub fn position(&self) -> Pos {
        self.p
    }

    pub fn velocity(&self) -> Velocity {
        self.v
    }

    pub fn next(&self) -> Probe {
        Probe {
            p: self.p + self.v,
            v: self.v.next(),
        }
    }

    pub fn fire(&self) -> ProbeFlight {
        ProbeFlight(self.p, self.v)
    }

    pub fn fire_at<'a>(&self, target: &'a Target) -> ProbeFlightTargetted<'a> {
        ProbeFlightTargetted { flight: self.fire(), target, done: false }
    }

    pub fn find_highest_trajectory(target: &Target) -> Option<Velocity> {
        // assuming target is always towards positive x
        let x_v = iter::successors(Some(1), |n| Some(n+1))
            .map(|v| (v * (v+1) / 2, v))
            .find(|(d, _)| d >= target.x.start())
            .unwrap()
            .1;

        // assuming target is always down
        let y_diff = target.y.start() + 1;

        Some(Velocity::from((x_v, -y_diff)))
    }
}

impl From<Velocity> for Probe {
    fn from(v: Velocity) -> Self {
        Probe { p: (0, 0).into(), v }
    }
}

impl fmt::Debug for Probe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?}", self.position(), self.velocity())
    }
}

impl Pos {
    fn x(&self) -> Int { self.0 }
    fn y(&self) -> Int { self.1 }
}

impl From<(Int, Int)> for Pos {
    fn from((x, y): (Int, Int)) -> Self {
        Self(x, y)
    }
}

impl Add<Velocity> for Pos
{
    type Output = Self;
    fn add(self, v: Velocity) -> Self::Output {
        Self(self.x() + v.x(), self.y() + v.y())
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x(), self.y())
    }
}

impl Velocity {
    pub const fn x(&self) -> i32 { self.0 }
    pub const fn y(&self) -> i32 { self.1 }

    pub fn next(&self) -> Self {
        Self(self.x() + (-self.x()).clamp(-1, 1), self.y() - 1)
    }
}

impl From<(Int, Int)> for Velocity {
    fn from((x, y): (Int, Int)) -> Self {
        Self(x, y)
    }
}

impl fmt::Debug for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "+({},{})", self.x(), self.y())
    }
}

pub struct ProbeFlight(Pos, Velocity);
impl Iterator for ProbeFlight {
    type Item = Probe;
    fn next(&mut self) -> Option<Self::Item> {
        self.0 = self.0 + self.1;
        self.1 = self.1.next();
        Some(Self::Item { p: self.0, v: self.1 })
    }
}

pub enum Flight {
    Flying(Probe),
    Hit(Probe),
    Missed(Probe),
}

pub struct ProbeFlightTargetted<'a> {
    flight: ProbeFlight,
    target: &'a Target,
    done: bool,
}
impl<'a> Iterator for ProbeFlightTargetted<'a> {
    type Item = Flight;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None
        }

        Some(match self.flight.next().unwrap() {
            p if self.target.contains(p.position()) => {
                self.done = true;
                Flight::Hit(p)
            },
            p if self.target.missed_by(&p) => {
                self.done = true;
                Flight::Missed(p)
            },
            p => Flight::Flying(p),
        })
    }
}

#[derive(Clone)]
pub struct Target {
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
}

impl Target {
    pub fn contains(&self, p: Pos) -> bool {
        self.x.contains(&p.x()) && self.y.contains(&p.y())
    }

    pub fn missed_by(&self, probe: &Probe) -> bool {
        let pos = probe.position();
        (pos.y() < *self.y.start())
            || match probe.velocity().x() {
                0 => !self.x.contains(&pos.x()),
                x if x < 0 => pos.x() < *self.x.start(),
                x if x > 0 => pos.x() > *self.x.end(),
                _ => false,
            }
    }
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={:?} y={:?}", self.x, self.y)
    }
}

impl<'a> TryFrom<&'a str> for Target {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Target, Self::Error> {
        // target area: x=20..30, y=-10..-5
        let pos_x_start = "target area: x=".len();
        let pos_x_end = s.find(|c| c == ',').unwrap();

        let x_str = &s[pos_x_start..pos_x_end];
        let y_str = &s[pos_x_end+4..];

        let x = parse_range::<i32>(x_str)?;
        let y = parse_range::<i32>(y_str)?;

        Ok(Target { x, y })
    }
}

fn parse_range<F>(s: &str) -> Result<RangeInclusive<F>, &'static str>
where F: FromStr
{
    // Assuming input string is always given in increasing order
    let p1_end = s.find(|c| c == '.').ok_or("no .. separator found")?;
    let start = s[0..p1_end].parse::<F>().map_err(|_| "invalid start")?;
    let end = s[p1_end+2..].parse::<F>().map_err(|_| "invalid end")?;
    Ok(start..=end)
}

fn p1(target: &Target) {
    let v = Probe::find_highest_trajectory(&target).unwrap();
    let probe = Probe::from(v);
    println!("{:?}", &probe);

    let mut max = 0;
    for tick in probe.fire_at(target) {
        match tick {
            Flight::Flying(p) => {
                max = cmp::max(max, p.position().y());
                println!("{:?}", p)
            },
            Flight::Hit(p) => println!("HIT! {:?}", p.position()),
            Flight::Missed(p) => println!("MISSED! {:?}", p.position()),
        }
    }

    println!("max height: {}", max);
}

fn main() {
    let stdin = io::stdin();
    let l = stdin.lock().lines().next().unwrap().unwrap();
    let target = Target::try_from(l.as_str()).unwrap();

    println!("target: {:?}", &target);

    p1(&target);
}