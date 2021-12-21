use std::cmp;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::iter::Extend;
use std::ops::{Index, Range};

type Int = i32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos([Int;2]);

impl Pos {
    pub fn x(&self) -> Int { self.0[0] }
    pub fn y(&self) -> Int { self.0[1] }
}

impl AsRef<[Int;2]> for Pos {
    fn as_ref(&self) -> &[Int;2] {
        &self.0
    }
}

impl From<[Int;2]> for Pos {
    fn from(a: [Int;2]) -> Self {
        Pos(a)
    }
}

impl From<&[Int;2]> for Pos {
    fn from(a: &[Int;2]) -> Self {
        Pos(*a)
    }
}

impl From<[usize;2]> for Pos {
    fn from(a: [usize;2]) -> Self {
        Pos([a[0] as Int, a[1] as Int])
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    Dark,
    Light,
}

impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pixel::Light => write!(f, "#"),
            Pixel::Dark => write!(f, "."),
        }
    }
}

pub struct Enhancer([Pixel; 512]);

impl Enhancer {
    pub fn try_from_str<S: AsRef<str>>(s: S) -> Result<Self, &'static str> {
        let s = s.as_ref();
        assert_eq!(s.len(), 512);

        let mut a = [Pixel::Dark; 512];
        for (i,c) in s.char_indices() {
            a[i] = match c {
                '#' => Pixel::Light,
                '.' => Pixel::Dark,
                _ => return Err("Invalid pixel character"),
            }
        }

        Ok(Enhancer(a))
    }
}

pub struct Image {
    points: HashSet<Pos>,
    dim: Dimensions,
    inf: Pixel,
}

impl Image {
    pub fn new() -> Self {
        Image { points: HashSet::new(), dim: Dimensions::new(), inf: Pixel::Dark }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dim.clone()
    }

    fn dimensions_of(points: &HashSet<Pos>) -> Dimensions {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for p in points {
            let x = p.x();
            let y = p.y();
            min_x = cmp::min(min_x, x);
            max_x = cmp::max(max_x, x);
            min_y = cmp::min(min_y, y);
            max_y = cmp::max(max_y, y);
        }

        Dimensions { x: min_x..max_x+1, y: min_y..max_y+1 }
    }

    pub fn count_lit(&self) -> usize {
        self.points.len()
    }

    pub fn infinity(&self) -> Pixel {
        self.inf
    }

    pub fn enhance(&mut self, enhancer: &Enhancer) {
        let mut next: HashSet<Pos> = HashSet::with_capacity(self.points.len());
        let dimensions = self.dimensions();
        let dim_x = dimensions.x();
        let dim_y = dimensions.y();

        for y in dim_y.start-1..dim_y.end+1 {
            for x in dim_x.start-1..dim_x.end+1 {
                let p = Pos::from([x, y]);
                if self.enhanced_pixel(p, enhancer) == Pixel::Light {
                    next.insert(p);
                };
            }
        }

        self.points.clear();
        self.points.extend(next);

        self.inf = enhancer.0[
            match self.inf {
                Pixel::Light => 0b111111111,
                Pixel::Dark => 0b000000000,
            }];

        self.dim = Self::dimensions_of(&self.points);
    }

    pub fn enhanced_pixel(&self, p: Pos, enhancer: &Enhancer) -> Pixel {
        enhancer.0[self.enhancer_index(p)]
    }

    pub fn enhancer_index(&self, p: Pos) -> usize {
        let mut idx = 0;
        for y in p.y()-1..=p.y()+1 {
            for x in p.x()-1..=p.x()+1 {
                idx = (idx << 1) | (match self[Pos::from([x, y])] {
                    Pixel::Light => 1,
                    Pixel::Dark => 0,
                });
            }
        }
        //println!("{},{} -> {:09b}", p.x(), p.y(), idx);
        idx
    }
}

impl From<HashSet<Pos>> for Image {
    fn from(points: HashSet<Pos>) -> Self {
        let dim = Image::dimensions_of(&points);
        Image { points, dim, inf: Pixel::Dark }
    }
}

impl Index<Pos> for Image {
    type Output = Pixel;
    fn index(&self, p: Pos) -> &Self::Output {
        if self.dimensions().contains(p) {
            match self.points.contains(&p) {
                true => &Pixel::Light,
                false => &Pixel::Dark,
            }
        } else {
            &self.inf
        }
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dimensions = self.dimensions();
        let dim_y = dimensions.y();
        let dim_x = dimensions.x();

        for y in dim_y.start-1..dim_y.end+1 {
            for x in dim_x.start-1..dim_y.end+1 {
                write!(f, "{}", self[Pos::from([x, y])])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Dimensions {
    x: Range<i32>,
    y: Range<i32>,
}

impl Dimensions {
    pub fn new() -> Dimensions {
        Dimensions { x: 0..0, y: 0..0 }
    }

    pub fn x(&self) -> Range<i32> {
        self.x.clone()
    }

    pub fn y(&self) -> Range<i32> {
        self.y.clone()
    }

    pub fn contains(&self, p: Pos) -> bool {
        self.x().contains(&p.x()) && self.y().contains(&p.y())
    }
}

impl fmt::Debug for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={:?} y={:?}", self.x(), self.y())
    }
}

pub fn read_input(lines: &mut impl Iterator<Item = String>) -> (Enhancer, Image) {
    let l = lines.next().unwrap();
    let enhancer = Enhancer::try_from_str(l).unwrap();
    lines.next();

    let mut image_set = HashSet::with_capacity(300);
    for (y, s) in lines.enumerate() {
        for (x, c) in s.as_str().char_indices() {
            if c == '#' {
                image_set.insert(Pos::from([x as i32, y as i32]));
            }
        }
    }
    (enhancer, Image::from(image_set))
}

fn main() {
    let stdin = io::stdin();
    let lines = &mut stdin.lock().lines().map(|l| l.unwrap());
    let (enhancer, mut image) = read_input(lines);
    //println!("{:?}", &enhancer.0);

    println!("dim: {:?}  inf: {}", image.dimensions(), image.infinity());
    println!("{}", image);

    /*
    image.enhance(&enhancer);
    println!("dim: {:?}  inf: {}", image.dimensions(), image.infinity());
    println!("{}", image);

    image.enhance(&enhancer);
    println!("dim: {:?}  inf: {}", image.dimensions(), image.infinity());
    println!("{}", image);
    */
    for _ in 0..50 {
        image.enhance(&enhancer);
    }
    println!("dim: {:?}  inf: {}", image.dimensions(), image.infinity());
    println!("{}", image);
    println!("lit: {}", image.count_lit());
}
