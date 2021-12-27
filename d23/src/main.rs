use std::cmp::{Ord, Ordering};
use std::hash::Hash;
use std::fmt;
use std::collections::HashSet;

use petgraph;
use petgraph::algo::astar;
use petgraph::visit;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}

const ALL_AMPHIPOD_TYPES: &[Amphipod] = &[Amphipod::Amber, Amphipod::Bronze, Amphipod::Copper, Amphipod::Desert];

pub type Room = Amphipod;
pub type RoomPos = u8;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    Hallway(u8),
    Room(Room, RoomPos),
}
impl Position {
    pub fn hallway_pos(&self) -> u8 {
        match self {
            Position::Hallway(n) => *n,
            Position::Room(Room::Amber, _) => 2,
            Position::Room(Room::Bronze, _) => 4,
            Position::Room(Room::Copper, _) => 6,
            Position::Room(Room::Desert, _) => 8,
        }
    }

    pub fn into_hallway(&self) -> Position {
        match self {
            Position::Hallway(_) => *self,
            Position::Room(Room::Amber, _) => Position::Hallway(2),
            Position::Room(Room::Bronze, _) => Position::Hallway(4),
            Position::Room(Room::Copper, _) => Position::Hallway(6),
            Position::Room(Room::Desert, _) => Position::Hallway(8),
        }
    }
}
impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else {
            match (self, other) {
                (Position::Room(_,_), Position::Hallway(_)) => Ordering::Less,
                (Position::Hallway(me), Position::Hallway(other)) => me.cmp(other),
                (Position::Room(ar, ad), Position::Room(br, bd)) if ar == br => bd.cmp(ad),
                (Position::Room(ar, _), Position::Room(br, _)) => ar.cmp(br),
                _ => Ordering::Greater,
            }
        }
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Amphipod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Amphipod::Amber => 'A',
            Amphipod::Bronze => 'B',
            Amphipod::Copper => 'C',
            Amphipod::Desert => 'D',
        })
    }
}
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Position::Room(a, d) => write!(f, "{}{}", a, d),
            Position::Hallway(n) => write!(f, "={}=", n),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct State([Position; 8]);

pub trait RoomSize {
    fn room_size() -> usize;
}

pub trait SliceBackedBurrow {
    fn stride() -> usize;
    fn positions_slice<'a>(&'a self, a: Amphipod) -> &'a [Position];
    fn positions_mut<'a>(&'a mut self, a: Amphipod) -> &'a mut [Position];
}

pub trait BurrowState: RoomSize {
    fn room_size() -> usize;

    fn is_goal(&self) -> bool;

    fn get(&self, pos: &Position) -> Option<Amphipod>;

    fn apply_movement<B: BurrowState + Copy>(&mut self, t: &StateTransition<B>);

    fn occupied(&self, pos: &Position) -> bool {
        self.get(pos).is_some()
    }

    fn min_energy(&self) -> Energy;

    fn is_blocked(&self, a: Amphipod, path: &Path) -> bool {
        match path.end() {
            Position::Room(rm, _) if !self.can_enter_room(a, rm) => false,
            _ => path.walk().skip(1).any(|p| self.occupied(&p)),
        }
    }

    fn can_enter_room(&self, ap: Amphipod, room: Room) -> bool {
        if room == ap {
            (0..<Self as BurrowState>::room_size())
                .all(|d| match self.get(&Position::Room(room, d as u8)) {
                    Some(a) if a == ap => true,
                    None => true,
                    _ => false
                })
        } else {
            false
        }
    }
}

impl<B> BurrowState for B
where B: SliceBackedBurrow + RoomSize + AsRef<[Position]> + Copy
{
    fn room_size() -> usize {
        Self::stride()
    }

    fn get(&self, pos: &Position) -> Option<Amphipod> {
        match self.as_ref().iter().position(|p| *p == *pos) {
            Some(n) => Some(match n / Self::room_size() {
                0 => Amphipod::Amber,
                1 => Amphipod::Bronze,
                2 => Amphipod::Copper,
                3 => Amphipod::Desert,
                _ => panic!("Out of bound when searching through positions"),
            }),
            None => None,
        }
    }

    fn is_goal(&self) -> bool {
        ALL_AMPHIPOD_TYPES
            .iter()
            .all(|&a| self.positions_slice(a)
                .iter()
                .all(|p| match p { Position::Room(rm, _) => *rm == a, _ => false}))
    }

    fn min_energy(&self) -> Energy {
        ALL_AMPHIPOD_TYPES.iter()
            .flat_map(move |ap| self.positions_slice(*ap)
                .into_iter()
                .map(move |p| match (*ap, p) {
                    (_, Position::Room(rm, _)) if ap == rm => 0,
                    (_, p) => Path::from([*p, Position::Room(*ap, 0)]).cost(*ap),
                }))
            .sum()
    }

    fn apply_movement<S>(&mut self, t: &StateTransition<S>) where S: BurrowState + Copy {
        let path = t.path;
        let pos = self.positions_mut(t.a);
        match pos.iter_mut().find(|p| **p == path.start()) {
            Some(e) => *e = path.end(),
            None => panic!("Not a valid movement"),
        };
        pos.sort();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Burrow2([Position; 8]);
impl Default for Burrow2 {
    fn default() -> Self { Burrow2([Position::Hallway(0); 8]) }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Burrow4([Position; 16]);
impl Default for Burrow4 {
    fn default() -> Self { Burrow4([Position::Hallway(0); 16]) }
}

impl RoomSize for Burrow2 {
    fn room_size() -> usize { 2 }
}
impl AsRef<[Position]> for Burrow2 {
    fn as_ref(&self) -> &[Position] { &self.0 }
}
impl AsMut<[Position]> for Burrow2 {
    fn as_mut(&mut self) -> &mut [Position] { &mut self.0 }
}

impl RoomSize for Burrow4 {
    fn room_size() -> usize { 4 }
}
impl AsRef<[Position]> for Burrow4 {
    fn as_ref(&self) -> &[Position] { &self.0 }
}
impl AsMut<[Position]> for Burrow4 {
    fn as_mut(&mut self) -> &mut [Position] { &mut self.0 }
}

impl<B> SliceBackedBurrow for B
where B: AsRef<[Position]> + AsMut<[Position]> + RoomSize
{
    fn stride() -> usize {
        B::room_size()
    }

    fn positions_slice(&self, a: Amphipod) -> &[Position] {
        let stride = Self::stride();
        let v = self.as_ref();
        match a {
            Amphipod::Amber => &v[0*stride..1*stride],
            Amphipod::Bronze => &v[1*stride..2*stride],
            Amphipod::Copper => &v[2*stride..3*stride],
            Amphipod::Desert => &v[3*stride..4*stride],
        }
    }

    fn positions_mut(&mut self, a: Amphipod) -> &mut [Position] {
        let stride = Self::stride();
        match a {
            Amphipod::Amber => &mut self.as_mut()[0*stride..1*stride],
            Amphipod::Bronze => &mut self.as_mut()[1*stride..2*stride],
            Amphipod::Copper => &mut self.as_mut()[2*stride..3*stride],
            Amphipod::Desert => &mut self.as_mut()[3*stride..4*stride],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path([Position; 2]);

impl Path {
    pub fn start(&self) -> Position { self.0[0] }
    pub fn end(&self) -> Position { self.0[1] }

    pub fn walk(&self) -> PathWalk {
        PathWalk { here: Some(self.start()), end: self.end() }
    }

    pub fn steps(&self) -> usize {
        self.walk().count() - 1
    }

    pub fn cost(&self, a: Amphipod) -> Energy {
        (self.steps() as Energy) * match a {
            Amphipod::Amber => 1,
            Amphipod::Bronze => 10,
            Amphipod::Copper => 100,
            Amphipod::Desert => 1000,
        }
    }
}

impl From<[Position; 2]> for Path {
    fn from(a: [Position; 2]) -> Path { Path(a) }
}
impl AsRef<[Position; 2]> for Path {
    fn as_ref(&self) -> &[Position; 2] { &self.0 }
}

pub struct PathWalk {
    here: Option<Position>,
    end: Position,
}
impl Iterator for PathWalk {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self.here {
            Some(p) if p == self.end => {
                self.here = None;
                Some(p)
            },
            Some(p) => {
                self.here = match (p, self.end) {
                    (Position::Hallway(n), Position::Room(a, _)) => {
                        let rm = Position::Room(a, 0);
                        if let Position::Hallway(t) = rm.into_hallway() {
                            if n == t {
                                Some(rm)
                            } else {
                                Some(Position::Hallway(if n < t { n + 1 } else { n - 1 }))
                            }
                        } else {
                            panic!();
                        }
                    },
                    (Position::Hallway(a), Position::Hallway(b))
                        => Some(Position::Hallway(if a < b { a + 1 } else { a - 1 })),
                    (Position::Room(aa, ad), Position::Room(ba, bd)) if aa == ba
                        => Some(Position::Room(aa, if ad > bd { ad - 1 } else { ad + 1 })),
                    (Position::Room(a, d), _) if d == 0 => Some(Position::Room(a, d).into_hallway()),
                    (Position::Room(a, d), _) => Some(Position::Room(a, d - 1)),
                };
                Some(p)
            }
            None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct StateGraph<B>(std::marker::PhantomData<B>)
where B: BurrowState + Copy + Eq + Default;

impl<B> visit::IntoEdges for StateGraph<B>
where B: BurrowState + SliceBackedBurrow + Clone + Copy + Eq + Default + Hash {
    type Edges = StateTransitions<B>;
    fn edges(self, state: B) -> Self::Edges {
        // Generate all possible state transitions
        let mut transitions = Vec::with_capacity(8);

        for &a in ALL_AMPHIPOD_TYPES {
            // Can amphipods go home?
            let room_is_clear = state.can_enter_room(a, a);

            // TODO: This requires SliceBackedBurrow
            for p in state.positions_slice(a) {
                match p {
                    // Already home
                    Position::Room(rm, _) if room_is_clear && *rm == a => (),
                    p if room_is_clear => {
                        // Find deepest room spot and go there
                        let target = (0..<B as BurrowState>::room_size()).rev()
                            .find_map(|d| {
                                let target_pos = Position::Room(a, d as u8);
                                if state.occupied(&target_pos) {
                                    None
                                } else {
                                    Some(target_pos)
                                }
                            }).unwrap();
                        
                        let path = [*p, target].into();
                        if !state.is_blocked(a, &path) {
                            transitions.push(StateTransition { start: state, a, path });
                        }
                    },
                    Position::Room(..) => {
                        // Go to all the hallway spots
                        for h in [0, 1, 3, 5, 7, 9, 10] {
                            let path = [*p, Position::Hallway(h)].into();
                            if !state.is_blocked(a, &path) {
                                transitions.push(StateTransition { start: state, a, path });
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
        transitions.into()
    }
}
impl<B> visit::IntoEdgeReferences for StateGraph<B> where B: BurrowState + Copy + Eq + Default + Hash {
    type EdgeRef = StateTransition<B>;
    type EdgeReferences = std::iter::Empty<Self::EdgeRef>;

    fn edge_references(self) -> Self::EdgeReferences {
        panic!("Not expecting to have all edges enumerated");
    }
}
impl<B> visit::IntoNeighbors for StateGraph<B> where B: BurrowState + Copy + Eq + Default {
    type Neighbors = std::iter::Empty<Self::NodeId>;

    fn neighbors(self, _start: Self::NodeId) -> Self::Neighbors {
        panic!("Unspected iteration of node neighbours");
    }
}

impl<B> visit::Visitable for StateGraph<B> where B: BurrowState + Copy + Eq + Default + Hash {
    type Map = HashSet<B>;

    fn visit_map(&self) -> Self::Map {
        HashSet::new()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl<B> visit::Data for StateGraph<B> where B: BurrowState + Copy + Eq + Default {
    type NodeWeight = ();
    type EdgeWeight = Energy;
}
impl<B> visit::GraphBase for StateGraph<B> where B: BurrowState + Copy + Eq + Default {
    type EdgeId = ();
    type NodeId = B;
}
impl<B> visit::GraphRef for StateGraph<B> where B: BurrowState + Copy + Eq + Default {}

pub struct StateTransitions<B> where B: BurrowState + Copy{
    transitions: Vec<StateTransition<B>>,
    n: usize,
}
impl<B> From<Vec<StateTransition<B>>> for StateTransitions<B> where B: BurrowState + Copy {
    fn from(transitions: Vec<StateTransition<B>>) -> Self {
        StateTransitions { transitions, n: 0 }
    }
}
impl<B> Iterator for StateTransitions<B> where B: BurrowState + Copy {
    type Item = StateTransition<B>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.transitions.get(self.n) {
            Some(&p) => { self.n += 1; Some(p) },
            None => None
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateTransition<B> where B: BurrowState + Copy {
    start: B,
    a: Amphipod,
    path: Path,
}

pub type Energy = u32;

impl<B> StateTransition<B> where B: BurrowState + Copy {
    pub fn cost(&self) -> Energy {
        self.path.cost(self.a)
    }
}

impl<B> visit::EdgeRef for StateTransition<B> where B: BurrowState + Copy + Eq + Hash {
    type NodeId = B;
    type EdgeId = ();
    type Weight = Energy;

    fn source(&self) -> Self::NodeId { self.start }
    fn target(&self) -> Self::NodeId {
        let mut target = self.start.clone();
        target.apply_movement(self);
        target
    }

    fn weight(&self) -> &Self::Weight { panic!() }
    fn id(&self) -> Self::EdgeId {}
}

// #############
// #...........#
// ###B#C#B#D###
//   #A#D#C#A#
//   #########
const _SAMPLE_INPUT: [Position; 8] = [
    Position::Room(Room::Amber, 1),
    Position::Room(Room::Desert, 1),
    Position::Room(Room::Amber, 0),
    Position::Room(Room::Copper, 0),
    Position::Room(Room::Bronze, 0),
    Position::Room(Room::Copper, 1),
    Position::Room(Room::Bronze, 1),
    Position::Room(Room::Desert, 0),
];

// #############
// #...........#
// ###B#B#C#D###
//   #D#C#A#A#
//   #########
const _PROBLEM_INPUT: [Position; 8] = [
    Position::Room(Room::Copper, 1),
    Position::Room(Room::Desert, 1),
    Position::Room(Room::Amber, 0),
    Position::Room(Room::Bronze, 0),
    Position::Room(Room::Bronze, 1),
    Position::Room(Room::Copper, 0),
    Position::Room(Room::Amber, 1),
    Position::Room(Room::Desert, 0),
];

impl From<&[Position]> for Burrow2 {
    fn from(p: &[Position]) -> Burrow2 {
        let mut d = [Position::Hallway(0); 8];
        d.clone_from_slice(p);
        Burrow2(d)
    }
}
impl fmt::Debug for Burrow2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for n in 0..=10 {
            match self.get(&Position::Hallway(n)) {
                Some(a) => write!(f, "{}", a)?,
                None => write!(f, ".")?,
            }
        }
        writeln!(f)?;
        for d in 0..2 {
            write!(f, "  ")?;
            for a in [Amphipod::Amber, Amphipod::Bronze, Amphipod::Copper, Amphipod::Desert] {
                match self.get(&Position::Room(a, d)) {
                    Some(a) => write!(f, "{} ", a)?,
                    None => write!(f, ". ")?,
                }
            }
            writeln!(f)?
        }
        write!(f, "")
    }
}

impl From<&[Position]> for Burrow4 {
    fn from(p: &[Position]) -> Burrow4 {
        let mut d = [Position::Hallway(0); 16];
        for i in 0..4 {
            d[i*4..i*4+2].clone_from_slice(&p[i*2..i*2+2]);
            for n in 0..2 {
                let idx = i*4+n;
                if let Position::Room(rm, r) = d[idx] {
                    if r == 1 {
                        d[idx] = Position::Room(rm, 3);
                    }
                }
            }
        }

        // INSERT:
        //   #D#C#B#A#
        //   #D#B#A#C#
        d[2] = Position::Room(Room::Copper, 2);
        d[3] = Position::Room(Room::Desert, 1);
        d[6] = Position::Room(Room::Bronze, 2);
        d[7] = Position::Room(Room::Copper, 1);
        d[10] = Position::Room(Room::Bronze, 1);
        d[11] = Position::Room(Room::Desert, 2);
        d[14] = Position::Room(Room::Amber, 1);
        d[15] = Position::Room(Room::Amber, 2);
        Burrow4(d)
    }
}
impl fmt::Debug for Burrow4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for n in 0..=10 {
            match self.get(&Position::Hallway(n)) {
                Some(a) => write!(f, "{}", a)?,
                None => write!(f, ".")?,
            }
        }
        writeln!(f)?;
        for d in 0..4 {
            write!(f, "  ")?;
            for a in [Amphipod::Amber, Amphipod::Bronze, Amphipod::Copper, Amphipod::Desert] {
                match self.get(&Position::Room(a, d)) {
                    Some(a) => write!(f, "{} ", a)?,
                    None => write!(f, ". ")?,
                }
            }
            writeln!(f)?
        }
        write!(f, "")
    }
}


fn find_shortest<B>(start: &B) -> Option<(Energy, Vec<B>)>
where B: BurrowState + SliceBackedBurrow + Copy + Eq + Default + std::hash::Hash {
    astar(StateGraph::<B>::default(), *start,
        |s| s.is_goal(),
        |m| m.cost(),
        |s| s.min_energy())
}

fn main() {
    println!("for SAMPLE");
    let burrow = Burrow2::from(_SAMPLE_INPUT.as_ref());
    println!("{:?}", &burrow);
    match find_shortest(&burrow) {
        Some((cost, states)) => {
            for s in states {
                println!(": {:?}", s.0);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };

    println!("for PROBLEM");
    let burrow = Burrow2::from(_PROBLEM_INPUT.as_ref());
    match find_shortest(&burrow) {
        Some((cost, states)) => {
            for s in states {
                //println!("{:?}", s);
                println!(": {:?}", s.0);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };

    println!("for SAMPLE (p2)");
    let burrow = Burrow4::from(_SAMPLE_INPUT.as_ref());
    println!("{:?}", &burrow);
    match find_shortest(&burrow) {
        Some((cost, states)) => {
            for s in states {
                println!(": {:?}", s.0);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };

    println!("for PROBLEM (p2)");
    let burrow = Burrow4::from(_PROBLEM_INPUT.as_ref());
    println!("{:?}", &burrow);
    match find_shortest(&burrow) {
        Some((cost, states)) => {
            for s in states {
                println!(": {:?}", s.0);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn wins() {
        // #############
        // #.....D.D.A.#
        // ###.#B#C#.###
        //   #A#B#C#.#
        //   #########
        let ALMOST_WIN = State([
            Position::Room(Amphipod::Amber, Room::Inner),
            Position::Hallway(9),
            Position::Room(Amphipod::Bronze, Room::Inner),
            Position::Room(Amphipod::Bronze, Room::Outer),
            Position::Room(Amphipod::Copper, Room::Inner),
            Position::Room(Amphipod::Copper, Room::Outer),
            Position::Hallway(5),
            Position::Hallway(7),
        ]);
        let energy = find_shortest(&ALMOST_WIN).unwrap().0;
        assert_eq!(7008, energy);
    }

    #[test]
    fn rooms_sort_before_hallway() {
        let h = Position::Hallway(5);
        let r = Position::Room(Amphipod::Amber, Room::Outer);
        println!("{}", match r.cmp(&h) {
            Ordering::Less => "<",
            Ordering::Greater => ">",
            Ordering::Equal => "=",
        });
        assert!(r < h);
    }
    */

    #[test]
    fn room_to_room_has_correct_steps() {
        let p = Path::from([Position::Room(Room::Amber, 1), Position::Room(Room::Bronze, 0)]);
        assert_eq!(p.walk().take(20).count(), 6);
    }
}
