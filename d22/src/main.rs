use std::cmp::{min, max};
use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;

type ReactorIx = i32;
type ReactorRange = RangeInclusive<ReactorIx>;

#[derive(Clone, PartialEq, Eq)]
pub struct Cuboid {
    x: ReactorRange,
    y: ReactorRange,
    z: ReactorRange,
}

impl Cuboid {
    pub fn x(&self) -> ReactorRange { self.x.clone() }
    pub fn y(&self) -> ReactorRange { self.y.clone() }
    pub fn z(&self) -> ReactorRange { self.z.clone() }

    pub fn volume(&self) -> u64 {
        (self.x.end() - self.x.start() + 1).abs() as u64
            * (self.y.end() - self.y.start() + 1).abs() as u64
            * (self.z.end() - self.z.start() + 1).abs() as u64
    }

    pub fn intersection(&self, other: &Cuboid) -> Option<Cuboid> {
        let x_overlaps = self.x.contains(other.x.start()) || self.x.contains(other.x.end());
        let y_overlaps = self.y.contains(other.y.start()) || self.y.contains(other.y.end());
        let z_overlaps = self.z.contains(other.z.start()) || self.z.contains(other.z.end());
        if x_overlaps && y_overlaps && z_overlaps {
            Some(Cuboid {
                x: max(*self.x.start(), *other.x.start())..=min(*self.x.end(), *other.x.end()),
                y: max(*self.y.start(), *other.y.start())..=min(*self.y.end(), *other.y.end()),
                z: max(*self.z.start(), *other.z.start())..=min(*self.z.end(), *other.z.end()),
            })
        } else {
            None
        }
    }

    pub fn to_off(&self) -> Instruction {
        Instruction { state: CubeState::Off, cuboid: self.clone() }
    }

    pub fn to_on(&self) -> Instruction {
        Instruction { state: CubeState::On, cuboid: self.clone() }
    }

    pub fn try_range_from(s: &str) -> Result<ReactorRange, &'static str> {
        let start_end = s.find('.').ok_or("failed to find \"..\"")?;
        let start = s[0..start_end].parse::<ReactorIx>().or(Err("parse fail"))?;
        let end = s[start_end+2..].parse::<ReactorIx>().or(Err("parse fail"))?;
        Ok(start..=end)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CubeState {
    On,
    Off,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Instruction {
    state: CubeState,
    cuboid: Cuboid,
}

impl Instruction {
    pub fn cuboid<'a>(&'a self) -> &'a Cuboid { &self.cuboid }

    pub fn is_boot(&self) -> bool {
        let limit = -50..=50;
        let c = self.cuboid();
        limit.contains(c.x.start()) && limit.contains(c.x.end())
            && limit.contains(c.y.start()) && limit.contains(c.y.end())
            && limit.contains(c.z.start()) && limit.contains(c.z.end())
    }

    pub fn is_on(&self) -> bool {
        self.state == CubeState::On
    }

    pub fn is_off(&self) -> bool {
        self.state == CubeState::Off
    }

    //pub fn apply(instructions: &mut impl Iterator<Item=Instruction>) -> Vec<Instruction> {
    pub fn apply<'a, T>(instructions: T) -> Vec<Instruction>
    where T: 'a + IntoIterator<Item = &'a Instruction> {
        let mut output: Vec<Instruction> = Vec::with_capacity(1000);

        for i in instructions {
            let mut adds = Vec::with_capacity(1);
            if i.is_on() {
                adds.push(i.clone());
                for existing in &output {
                    match existing.cuboid().intersection(i.cuboid()) {
                        Some(c) if existing.is_on() => adds.push(c.to_off()),
                        Some(c) if existing.is_off() => adds.push(c.to_on()),
                        _ => (),
                    };
                }
            } else {
                for existing in &output {
                    match existing.cuboid().intersection(i.cuboid()) {
                        Some(c) if existing.is_on() => adds.push(c.to_off()),
                        Some(c) if existing.is_off() => adds.push(c.to_on()),
                        _ => (),
                    };
                }
            }

            output.extend(adds);

            println!("!! {} ({})", count_on(&output), &output.len());
        }

        output
    }
}

impl fmt::Display for Cuboid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={}..{},y={}..{},z={}..{}",
            self.x.start(), self.x.end(),
            self.y.start(), self.y.end(),
            self.z.start(), self.z.end())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}",
            match self.state { CubeState::On => "on", CubeState::Off => "off"},
            self.cuboid())
    }
}

impl TryFrom<&str> for Cuboid {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let y_start = s.find(",y=").ok_or("failed to find ,y=")?;
        let z_start = s.find(",z=").ok_or("failed to find ,z=")?;
        let x = Self::try_range_from(&s[2..y_start])?;
        let y = Self::try_range_from(&s[y_start+3..z_start])?;
        let z = Self::try_range_from(&s[z_start+3..])?;
        Ok(Cuboid { x, y, z })
    }
}

impl TryFrom<&str> for Instruction {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let sep = s.find(' ').ok_or("failed to find space")?;
        let state = match &s[0..sep] {
            "on" => CubeState::On,
            "off" => CubeState::Off,
            _ => return Err("invalid state"),
        };
        let cuboid = Cuboid::try_from(&s[sep+1..])?;
        Ok(Instruction { state, cuboid })
    }
}


struct Instructions<I> where I: Iterator<Item = String> {
    lines: I
}

impl<I> From<I> for Instructions<I>
where I: Iterator<Item = String>
{
    fn from(lines: I) -> Self {
        Self { lines }
    }
}

impl<I> Iterator for Instructions<I>
where I: Iterator<Item = String>
{
    type Item = Instruction;
    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(s) => Some(Instruction::try_from(s.as_str()).unwrap()),
            None => None,
        }
    }
}

fn _p1(instructions: &[Instruction]) {
    let mut reactor = vec![[[false; 101]; 101]; 101];
    println!("start...");
    for i in instructions.iter().filter(|&ist| ist.is_boot()) {
        for x in i.cuboid().x() {
            for y in i.cuboid().y() {
                for z in i.cuboid().z() {
                    let x = (x + 50) as usize;
                    let y = (y + 50) as usize;
                    let z = (z + 50) as usize;
                    reactor[x][y][z] = match i.state {
                        CubeState::On => true,
                        CubeState::Off => false,
                    }
                }
            }
        }
    }

    let on_count = reactor.iter()
        .map(|ys| ys.map(|zs| zs.iter().filter(|&&s| s).count()).iter().sum::<usize>())
        //.iter()
        .sum::<usize>();

    println!("ON: {}", on_count);
}

fn _p1v2(instructions: &Vec<Instruction>) {
    //let instructions: Vec<&Instruction> = instructions.iter().filter(|&i| i.is_boot()).collect();
    let instructions = instructions.iter().filter(|&i| i.is_boot());

    let applied = Instruction::apply(instructions);
    let sum = count_on(&applied);
    println!("result: {}", sum);
}

fn count_on(countable: &Vec<Instruction>) -> u64 {
    let mut sum = 0;
    for i in countable {
        let v = i.cuboid().volume();
        if i.is_on() {
            sum += v;
        } else {
            sum -= v;
        }
    }
    sum
}

fn _p2(instructions: &Vec<Instruction>) {
    let applied = Instruction::apply(instructions);
    let sum = count_on(&applied);
    println!("result: {}", sum);
}

fn main() {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap());

    let instructions: Vec<Instruction> = Instructions::from(lines).collect();

    for inst in &instructions {
        println!("{}", inst);
    }

    //_p1(instructions.as_slice());
    _p1v2(&instructions);

    //_p2(&instructions);
}
