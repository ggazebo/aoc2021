use std::io;
use std::io::BufRead;
use std::collections::HashMap;
use std::fmt;
use std::str;

pub type Element = u8;
pub type ElementCount = usize;

#[derive(Clone, PartialEq, Eq)]
pub struct Polymer(Vec<Element>);

impl Polymer {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn with_insertions(&self, map: &InsertionMap) -> Polymer {
        let mut next = Vec::with_capacity(self.len() * 2);

        for s in self.0.windows(2) {
            let a = s[0];
            let b = s[1];

            next.push(a);

            match map.get(&(a, b)) {
                Some(&e) => next.push(e),
                _ => (),
            };
        }
        next.push(*self.0.last().unwrap());

        Polymer(next)
    }

    pub fn from<S>(s: S) -> Polymer
    where S: AsRef<str>
    {
        Polymer(s.as_ref().bytes().collect())
    }

    pub fn tally(&self) -> HashMap<Element, ElementCount> {
        let mut map = HashMap::new();
        for &e in &self.0 {
            map.entry(e).and_modify(|count| *count += 1).or_insert(0);
        }
        map
    }
}

impl fmt::Display for Polymer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(self.0.as_slice()).unwrap())
    }
}

pub struct PolymerData(HashMap<(Element, Element), ElementCount>);

impl PolymerData {
    pub fn from(p: &Polymer) -> PolymerData {
        let mut counts = HashMap::new();
        counts.insert((0, p.0[0]), 1);
        counts.insert((*p.0.last().unwrap(), 0), 1);

        for s in p.0.windows(2) {
            let pair = (s[0], s[1]);
            counts.entry(pair).and_modify(|c| *c += 1).or_insert(1);
        }

        PolymerData(counts)
    }

    pub fn with_insertions(&self, map: &InsertionMap) -> PolymerData {
        let mut next = HashMap::with_capacity(self.0.len());

        for (&pair, &v) in &self.0 {
            match map.get(&pair) {
                Some(&e) => {
                    let (a, b) = pair;
                    next.entry((a, e)).and_modify(|c| *c += v).or_insert(v);
                    next.entry((e, b)).and_modify(|c| *c += v).or_insert(v);
                },
                None => { next.entry(pair).and_modify(|c| *c += v).or_insert(v); },
            }
        }

        PolymerData(next)
    }

    pub fn tally(&self) -> HashMap<Element, ElementCount> {
        let mut tally = HashMap::new();
        for (pair, &v) in &self.0 {
            for e in [pair.0, pair.1] {
                tally.entry(e).and_modify(|c| *c += v).or_insert(v);
            }
        }

        tally.remove(&0);

        for (_, v) in tally.iter_mut() {
            *v = *v / 2;
        }

        tally
    }
}

impl fmt::Debug for PolymerData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (pair, v) in &self.0 {
            write!(f, "({}):{} ", str::from_utf8(&[pair.0, pair.1]).unwrap(), v)?;
        }
        write!(f, "]")?;
        let tally = self.tally();
        write!(f, "{{ ")?;
        for (k, v) in tally {
            write!(f, "{}:{} ", str::from_utf8(&[k]).unwrap(), v)?;
        }
        write!(f, "}}")
    }
}

pub type InsertionMap = HashMap<(Element, Element), Element>;

fn parse_map(it: impl Iterator<Item = String>) -> InsertionMap {
    let mut map = InsertionMap::new();

    for s in it {
        let bytes = s.as_bytes();
        let a = bytes[0];
        let b = bytes[1];
        let insert = bytes[6];

        map.insert((a, b), insert);
    }

    map
}

fn main() {
    let stdin = io::stdin();
    let mut it = stdin.lock().lines().map(|l| l.unwrap());

    let seed = Polymer::from(it.next().unwrap().trim_end());
    it.next();
    let map = parse_map(it);


    //let mut next = seed;
    let mut next = PolymerData::from(&seed);
    println!("0: {} {:?}", &seed, &next);
    for _i in 1..=40 {
        next = next.with_insertions(&map);
        //println!("{}: {:?}", _i, next);
    }

    let tally = next.tally();
    let score = tally.values().max().unwrap_or(&0) - tally.values().min().unwrap_or(&0);
    println!("score: {}", score);
}
