use std::cmp;
use std::io;
use std::io::BufRead;
use std::fmt;
use itertools::Itertools;

type Leaf = Option<u8>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SnailfishNumber {
    //root: Root,
    l: [Leaf; 32],
}

impl SnailfishNumber {
    pub fn magnitude(&self) -> u32 {
        Self::magnitude_slice(&self.l).unwrap()
    }

    fn magnitude_slice(s: &[Option<u8>]) -> Option<u32> {
        if s.len() == 1 {
            match s[0] {
                Some(n) => Some(n as u32),
                None => None,
            }
        } else {
            let mid = s.len() / 2;
            let left = Self::magnitude_slice(&s[0..mid]);
            let right = Self::magnitude_slice(&s[mid..]);
            match (left, right) {
                (Some(l), Some(r)) => Some(3 * l as u32 + 2 * r as u32),
                (Some(l), None) => Some(l),
                (None,  _) => None,
            }
        }
    }

    pub fn reduce(&mut self) {
        loop {
            if self.explode() {
                continue;
            }
            if self.split() {
                continue;
            }
            break;
        }
    }

    fn explode(&mut self) -> bool {
        let mut left_idx: Option<usize> = None;
        let mut right_idx: Option<usize> = None;
        let mut left = None;
        let mut right = None;
        for (i, pair) in self.l.chunks_exact_mut(2).enumerate() {
            //println!("{:?} {:?}<-{:?},{:?}->{:?}", pair, left_idx, left, right, right_idx);
            match (pair[0], pair[1]) {
                (Some(l), Some(r)) if left.is_none() => {
                    left = Some(l);
                    right = Some(r);

                    pair[0] = Some(0);
                    pair[1] = None;
                },
                (Some(_), _) if left.is_none() => left_idx = Some(i*2),
                (_, Some(_)) if left.is_none() => left_idx = Some(i*2 + 1),
                (Some(_), _) if right.is_some() && right_idx.is_none() => {
                    right_idx = Some(i*2);
                    //println!("explode to: {:?}<-{:?},{:?}->{:?}", left_idx, left, right, right_idx);
                    break
                },
                (_, Some(_)) if right.is_some() && right_idx.is_none() => {
                    right_idx = Some(i*2 + 1);
                    //println!("explode to: {:?}<-{:?},{:?}->{:?}", left_idx, left, right, right_idx);
                    break;
                },
                _ => (),
            }
        };

        match (left, right) {
            (Some(l), Some(r)) => {
                if left_idx.is_some() {
                    let p = &mut self.l[left_idx.unwrap()];
                    *p = Some(p.unwrap() + l);
                }
                if right_idx.is_some() {
                    let i = right_idx.unwrap();
                    let p = &mut self.l[i];
                    *p = Some(p.unwrap() + r);
                }
                return true
            },
            _ => (),
        }
        false
    }

    fn split(&mut self) -> bool {
        let left = self.l.iter()
            .position(|n| match n {
                Some(v) if *v > 9 => true,
                _ => false,
            });

        let left = match left {
            Some(i) => i,
            None => return false,
        };

        let right = self.l[left+1..]
            .iter()
            .position(|n| n.is_some())
            .unwrap_or(self.l[left..].len()) + 1;
        let right = left + right / 2;

        let v = self.l[left].unwrap();
        self.l[left] = Some(v / 2);
        self.l[right] = Some((v + 1) / 2);

        true
    }

    fn write_tree(l: &[Leaf], f: &mut fmt::Formatter) -> fmt::Result {
        let is_bottom = l[1..].iter().all(|v| v.is_none());
        if is_bottom {
            return write!(f, "{}", l[0].unwrap())
        }

        let mid = l.len()/2;
        write!(f, "[")?;
        Self::write_tree(&l[0..mid], f)?;
        write!(f, ",")?;
        Self::write_tree(&l[mid..mid*2], f)?;
        write!(f, "]")
    }

    fn read_tree(a: &mut [Leaf], s: &str, width: usize) -> usize {
        let comma_pos = if s.starts_with("[[") {
            1 + Self::read_tree(a, &s[1..], width / 2)
        } else {
            let n = s.find(',').unwrap();
            a[0] = Some(s[1..n].parse::<u8>().unwrap());
            a[1..width].fill(None);
            n
        };

        let a = &mut a[width..];
        let right_s = &s[comma_pos+1..];
        //println!("!! {}", &right_s);
        let end = if right_s.starts_with('[') {
            Self::read_tree(a, right_s, width / 2)
        } else {
            let n = right_s.find(']').unwrap();
            a[0] = Some(right_s[0..n].parse::<u8>().unwrap());
            a[1..width].fill(None);
            n
        };

        comma_pos + 1 + end + 1
    }
}

impl std::ops::Add<SnailfishNumber> for SnailfishNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut l = [None; 32];

        let mut left = self.l.chunks_exact(2)
            .map(|c| c[0]);
        let mut right = rhs.l.chunks_exact(2)
            .map(|c| c[0]);

        l[0..16].fill_with(|| left.next().unwrap());
        l[16..].fill_with(|| right.next().unwrap());

        //println!("after sum: {:?}", &l);
        let mut sum = SnailfishNumber { l };
        sum.reduce();
        sum
    }
}

impl TryFrom<&str> for SnailfishNumber {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut l = [None; 32];
        Self::read_tree(&mut l, s, 16);
        Ok(SnailfishNumber { l })
    }
}

impl fmt::Display for SnailfishNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        SnailfishNumber::write_tree(&self.l[0..], f)
    }
}

impl fmt::Debug for SnailfishNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <SnailfishNumber as fmt::Display>::fmt(self, f)
    }
}

/*
type Root = SnailfishNumInner<SnailfishNumInner<SnailfishNumInner<SnailfishNumInner<u8>>>>;
type Nest1 = SnailfishNumInner<SnailfishNumInner<SnailfishNumInner<u8>>>;
type Nest2 = SnailfishNumInner<SnailfishNumInner<u8>>;
type Nest3 = SnailfishNumInner<u8>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SnailfishNumInner<T> {
    Pair(T, T),
    Literal(u8),
}

impl Root {
    fn reduce(&mut self) {

    }

    fn iter_leafs_mut<'a>(&'a mut self) -> SnailfishNumLeafs<'a> {
        SnailfishNumLeafs { num: self, n: 0 }
    }
}

impl<T> SnailfishNumInner<T>
where T: SnailfishNum
{
    fn left_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        match self {
            Self::Pair(n, _) => Some(n),
            _ => None,
        }
    }

    fn right_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        match self {
            Self::Pair(_, n) => Some(n),
            _ => None,
        }
    }
}

pub trait SnailfishNum {
    type Data;

    fn magnitude(&self) -> u32;
    fn as_pair(&self) -> Option<(&Self::Data, &Self::Data)>;
    fn as_literal(&self) -> Option<u8>;
}

impl<T> SnailfishNum for SnailfishNumInner<T>
where T: SnailfishNum
{
    type Data = T;

    fn magnitude(&self) -> u32 {
        match self {
            Self::Pair(a, b) => 3 * a.magnitude() + 2 * b.magnitude(),
            Self::Literal(v) => *v as u32,
        }
    }

    fn as_pair(&self) -> Option<(&Self::Data, &Self::Data)> {
        match self {
            Self::Pair(a, b) => Some((&a, &b)),
            _ => None,
        }
    }

    fn as_literal(&self) -> Option<u8> {
        match self {
            Self::Literal(v) => Some(*v),
            _ => None,
        }
    }
}

impl SnailfishNum for u8 {
    type Data = ();

    fn magnitude(&self) -> u32 { *self as u32 }
    fn as_pair(&self) -> Option<(&Self::Data, &Self::Data)> { None }
    fn as_literal(&self) -> Option<Self> { Some(*self) }
}

struct SnailfishNumLeafs<'a> {
    num: &'a Root,
    n: usize,
}

impl<'a> Iterator for SnailfishNumLeafs<'a> {
    type Item = &'a mut Nest3;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
*/


fn main() {
    let stdin = io::stdin();
    let nums: Vec<SnailfishNumber> = stdin.lock().lines()
        .map(|l| SnailfishNumber::try_from(l.unwrap().as_str()).unwrap())
        .collect();
    let sum = nums.iter().copied().reduce(|a, n| {
        let s = a + n;
        println!("{} + {} = {}", &a, &n, &s);
        s
    }).unwrap();
    println!("{}", &sum);
    println!("magnitude {}", sum.magnitude());

    let perms = nums.iter().permutations(2);
    let max_magnitude = perms
        .fold(0, |max, n| {
            let a = n[0];
            let b = n[1];
            let m = (*a + *b).magnitude();
            cmp::max(max, m)
        });

    println!("max sum {}", max_magnitude);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explode1() {
        let mut n = SnailfishNumber { l: [
            Some(9), Some(8),
            Some(1), None,
            Some(2), None, None, None,
            Some(3), None, None, None, None, None, None, None,
            Some(4), None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None ] };

        let exploded = n.explode();
        assert!(exploded);
        assert_eq!(n.l[0..4], [Some(0), None, Some(9), None])
    }

    #[test]
    fn expode_samples() {
        {
            let mut n = SnailfishNumber::try_from("[[[[[9,8],1],2],3],4]").unwrap();
            n.explode();
            assert_eq!(n, SnailfishNumber::try_from("[[[[0,9],2],3],4]").unwrap());
        }

        {
            let mut n = SnailfishNumber::try_from("[7,[6,[5,[4,[3,2]]]]]").unwrap();
            n.explode();
            assert_eq!(n, SnailfishNumber::try_from("[7,[6,[5,[7,0]]]]").unwrap());
        }

        {
            let mut n = SnailfishNumber::try_from("[[6,[5,[4,[3,2]]]],1]").unwrap();
            n.explode();
            assert_eq!(n, SnailfishNumber::try_from("[[6,[5,[7,0]]],3]").unwrap());
        }

        {
            let mut n = SnailfishNumber::try_from("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]").unwrap();
            n.explode();
            assert_eq!(n, SnailfishNumber::try_from("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]").unwrap());
        }

        {
            let mut n = SnailfishNumber::try_from("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]").unwrap();
            n.explode();
            assert_eq!(n, SnailfishNumber::try_from("[[3,[2,[8,0]]],[9,[5,[7,0]]]]").unwrap());
        }

        let mut n = SnailfishNumber::try_from("[[[[[1,1],[2,2]],[3,3]],[4,4]],[5,5]]").unwrap();
        n.explode();
        assert_eq!(n, SnailfishNumber::try_from("[[[[0,[3,2]],[3,3]],[4,4]],[5,5]]").unwrap());

        let mut n = SnailfishNumber::try_from("[[[[0,[3,2]],[3,3]],[4,4]],[5,5]]").unwrap();
        n.explode();
        assert_eq!(n, SnailfishNumber::try_from("[[[[3,0],[5,3]],[4,4]],[5,5]]").unwrap());
    }

    #[test]
    fn reduce_samples() {
        let mut n = SnailfishNumber::try_from("[[[[[1,1],[2,2]],[3,3]],[4,4]],[5,5]]").unwrap();
        n.reduce();
        assert_eq!(n, SnailfishNumber::try_from("[[[[3,0],[5,3]],[4,4]],[5,5]]").unwrap());
    }

    #[test]
    fn splits() {
        {
            let mut n = SnailfishNumber::try_from("[10,11]").unwrap();
            while n.split(){}
            assert_eq!(n, SnailfishNumber::try_from("[[5,5],[5,6]]").unwrap());
        }
    }

    #[test]
    fn sum_sample1() {
        let a = SnailfishNumber::try_from("[[[[4,3],4],4],[7,[[8,4],9]]]").unwrap();
        let b = SnailfishNumber::try_from("[1,1]").unwrap();

        let s = a + b;
        assert_eq!(s, SnailfishNumber::try_from("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]").unwrap());
    }

    #[test]
    fn mangnitude_samples() {
        assert_eq!(SnailfishNumber::try_from("[9,1]").unwrap().magnitude(), 29);
        assert_eq!(SnailfishNumber::try_from("[1,9]").unwrap().magnitude(), 21);
        assert_eq!(SnailfishNumber::try_from("[[9,1],[1,9]]").unwrap().magnitude(), 129);
        assert_eq!(SnailfishNumber::try_from("[[1,2],[[3,4],5]]").unwrap().magnitude(), 143);
        assert_eq!(SnailfishNumber::try_from("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]").unwrap().magnitude(), 1384);
        assert_eq!(SnailfishNumber::try_from("[[[[1,1],[2,2]],[3,3]],[4,4]]").unwrap().magnitude(), 445);
        assert_eq!(SnailfishNumber::try_from("[[[[3,0],[5,3]],[4,4]],[5,5]]").unwrap().magnitude(), 791);
        assert_eq!(SnailfishNumber::try_from("[[[[5,0],[7,4]],[5,5]],[6,6]]").unwrap().magnitude(), 1137);
        assert_eq!(SnailfishNumber::try_from("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]").unwrap().magnitude(), 3488);
    }

    #[test]
    fn sum_samples() {
        let adder = |inputs: &[&str]| {
            inputs
                .iter()
                .map(|&s| SnailfishNumber::try_from(s).unwrap())
                .reduce(|s, n| s + n)
                .unwrap()
        };

        let l = ["[1,1]", "[2,2]", "[3,3]", "[4,4]"];
        assert_eq!(adder(&l), SnailfishNumber::try_from("[[[[1,1],[2,2]],[3,3]],[4,4]]").unwrap());

        let l = ["[1,1]", "[2,2]", "[3,3]", "[4,4]", "[5,5]"];
        assert_eq!(adder(&l), SnailfishNumber::try_from("[[[[3,0],[5,3]],[4,4]],[5,5]]").unwrap());

        let l = ["[1,1]", "[2,2]", "[3,3]", "[4,4]", "[5,5]", "[6,6]"];
        assert_eq!(adder(&l), SnailfishNumber::try_from("[[[[5,0],[7,4]],[5,5]],[6,6]]").unwrap());
    }
}