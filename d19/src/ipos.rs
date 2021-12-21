use std::convert::{AsRef};
use std::fmt;
use std::num;
use std::ops::{Add, Mul, Sub};

pub type Int = i32;

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Pos([Int; 3]);

pub const ORIGIN: Pos = Pos([0; 3]);

impl Pos {
    pub fn dot(&self, other: Pos) -> Int {
        self.0.dot(other.0)
    }

    pub fn square(&self) -> Int {
        self.dot(*self)
    }

    pub fn rotate(&self, r: Rotation) -> Self {
        let m = r.0;
        Pos([
            self.dot(Pos::from(m[0])),
            self.dot(Pos::from(m[1])),
            self.dot(Pos::from(m[2]))
        ])
    }

    pub fn manhattan(&self, other: Self) -> Int {
        (0..3).map(|i| (other.0[i] - self.0[i]).abs()).sum()
    }
}

impl From<[Int; 3]> for Pos {
    fn from(a: [Int; 3]) -> Pos {
        Pos(a)
    }
}

impl TryFrom<&str> for Pos
{
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut splits = s.split(',');
        let mut v: [Int; 3] = [0; 3];
        v.fill_with(|| splits.next().unwrap().parse::<Int>().unwrap());
        Ok(Pos::from(v))
    }
}

impl AsRef<[Int; 3]> for Pos {
    fn as_ref<'a>(&'a self) -> &'a [Int; 3] {
        &self.0
    }
}

impl Add<Pos> for Pos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let a = self.0;
        let b = rhs.0;
        Pos([a[0]+b[0], a[1]+b[1], a[2]+b[2]])
    }
}

impl Mul<Pos> for Pos {
    type Output = Int;
    fn mul(self, rhs: Self) -> Self::Output {
        let a = self.0;
        let b = rhs.0;
        a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
    }
}

impl Mul<Int> for Pos {
    type Output = Pos;
    fn mul(self, c: Int) -> Self::Output {
        let a = self.0;
        Pos([a[0]*c, a[1]*c, a[2]*c])
    }
}

impl Sub<Pos> for Pos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let a = self.0;
        let b = rhs.0;
        Pos([a[0]-b[0], a[1]-b[1], a[2]-b[2]])
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = self.as_ref();
        write!(f, "{},{},{}", a[0], a[1], a[2])
    }
}
impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

trait Vec3 {
    fn dot(self, other: Self) -> Int;
    fn element_product(self, other: Self) -> Self;
}

impl Vec3 for [Int; 3] {
    fn dot(self, other: Self) -> Int {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }

    fn element_product(self, other: Self) -> Self {
        [self[0] * other[0], self[1] * other[1], self[2] * other[2]]
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rotation([[Int; 3]; 3]);

impl Rotation {
    pub fn all() -> AllRotations {
        AllRotations { n: 0}
    }

    pub fn chain(self, r: Rotation) -> Rotation {
        let a = self.0;
        let rt = r.transpose();
        let t = rt.0;
        /*
        Rotation([
            self.0[0].element_product(t.0[0]),
            self.0[1].element_product(t.0[1]),
            self.0[2].element_product(t.0[2])
        ])
        */
        Rotation([
            [a[0].dot(t[0]), a[0].dot(t[1]), a[0].dot(t[2])],
            [a[1].dot(t[0]), a[1].dot(t[1]), a[1].dot(t[2])],
            [a[2].dot(t[0]), a[2].dot(t[1]), a[2].dot(t[2])],
        ])
    }

    pub fn transpose(self) -> Rotation {
        let m = self.0;
        Rotation([
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ])
    }
}

impl fmt::Debug for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = self.0;
        let r = m[0];
        write!(f, "{},{},{}|", r[0], r[1], r[2])?;
        let r = m[1];
        write!(f, "{},{},{}|", r[0], r[1], r[2])?;
        let r = m[2];
        write!(f, "{},{},{}", r[0], r[1], r[2])
    }
}

pub struct AllRotations {
    n: u8,
}

impl Iterator for AllRotations {
    type Item = Rotation;

    fn next(&mut self) -> Option<Rotation> {
        let now = self.n;
        self.n += 1;
        Some(match now {
            0 => ROT_ID,
            1 => ROT_Y1,
            2 => ROT_Y2,
            3 => ROT_Y3,
            4 => ROT_X1,
            5 => ROT_X1.chain(ROT_Z1),
            6 => ROT_X1.chain(ROT_Z2),
            7 => ROT_X1.chain(ROT_Z3),
            8 => ROT_X2,
            9 => ROT_X2.chain(ROT_Y1),
            10 => ROT_X2.chain(ROT_Y2),
            11 => ROT_X2.chain(ROT_Y3),
            12 => ROT_X3,
            13 => ROT_X3.chain(ROT_Z1),
            14 => ROT_X3.chain(ROT_Z2),
            15 => ROT_X3.chain(ROT_Z3),
            16 => ROT_Z1,
            17 => ROT_Z1.chain(ROT_X1),
            18 => ROT_Z1.chain(ROT_X2),
            19 => ROT_Z1.chain(ROT_X3),
            20 => ROT_Z3,
            21 => ROT_Z3.chain(ROT_X1),
            22 => ROT_Z3.chain(ROT_X2),
            23 => ROT_Z3.chain(ROT_X3),
            _ => return None,
        })
    }
}

pub const ROT_ID: Rotation = Rotation([[1,0,0],[0,1,0],[0,0,1]]);
pub const ROT_X1: Rotation = Rotation([[1,0,0],[0,0,-1],[0,1,0]]);
pub const ROT_X2: Rotation = Rotation([[1,0,0],[0,-1,0],[0,0,-1]]);
pub const ROT_X3: Rotation = Rotation([[1,0,0],[0,0,1],[0,-1,0]]);
pub const ROT_Y1: Rotation = Rotation([[0,0,-1],[0,1,0],[1,0,0]]);
pub const ROT_Y2: Rotation = Rotation([[-1,0,0],[0,1,0],[0,0,-1]]);
pub const ROT_Y3: Rotation = Rotation([[0,0,1],[0,1,0],[-1,0,0]]);
pub const ROT_Z1: Rotation = Rotation([[0,1,0],[-1,0,0],[0,0,1]]);
pub const ROT_Z2: Rotation = Rotation([[-1,0,0],[0,-1,0],[0,0,1]]);
pub const ROT_Z3: Rotation = Rotation([[0,-1,0],[1,0,0],[0,0,1]]);