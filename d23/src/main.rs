use std::cmp;
use std::cmp::{Ord, Ordering};
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    Hallway(u8),
    Room(Amphipod, Room),
}
impl Position {
    pub fn hallway_pos(&self) -> u8 {
        match self {
            Position::Hallway(n) => *n,
            Position::Room(Amphipod::Amber, _) => 2,
            Position::Room(Amphipod::Bronze, _) => 4,
            Position::Room(Amphipod::Copper, _) => 6,
            Position::Room(Amphipod::Desert, _) => 8,
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
                (Position::Room(ar, Room::Inner), Position::Room(br, _)) if ar == br => Ordering::Less,
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
            Position::Room(a, Room::Inner) => write!(f, "{}.", a),
            Position::Room(a, Room::Outer) => write!(f, "{}|", a),
            Position::Hallway(n) => write!(f, "={}=", n),
        }
    }
}
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for n in 0..=10 {
            match self.occupied(Position::Hallway(n)) {
                Some(a) => write!(f, "{}", a)?,
                None => write!(f, ".")?,
            }
        }
        writeln!(f)?;
        for d in [Room::Outer, Room::Inner] {
            write!(f, "  ")?;
            for a in [Amphipod::Amber, Amphipod::Bronze, Amphipod::Copper, Amphipod::Desert] {
                match self.occupied(Position::Room(a, d)) {
                    Some(a) => write!(f, "{} ", a)?,
                    None => write!(f, ". ")?,
                }
            }
            writeln!(f)?
        }
        write!(f, "")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Room {
    Inner,
    Outer,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct State([Position; 8]);

impl State {
    pub fn is_goal(&self) -> bool {
        *self == GOAL
    }

    pub fn is_occupied(&self, position: Position) -> bool {
        self.0.iter().find(|p| **p == position).is_some()
    }

    pub fn occupied(&self, position: Position) -> Option<Amphipod> {
        match self.0.iter().position(|p| *p == position) {
            Some(idx) => Some(match idx / 2 {
                0 => Amphipod::Amber,
                1 => Amphipod::Bronze,
                2 => Amphipod::Copper,
                3 => Amphipod::Desert,
                _ => panic!(),
            }),
            None => None,
        }
    }

    pub fn is_blocked(&self, a: Amphipod, path: &Path) -> bool {
        if self.is_occupied(path.end()) {
            return true
        }

        // Check if room can be entered
        match path.as_ref() {
            // Don't try to move within the same room
            [Position::Room(from, _), Position::Room(to, _)] if from == to => return true,
            // Can't move out of Inner if Outer is occupied
            [Position::Room(a, Room::Inner), _] if self.is_occupied(Position::Room(*a, Room::Outer)) => return true,
            // Can't move into Inner if Outer is occupied
            [_, Position::Room(r_type, Room::Inner)] => {
                if self.is_occupied(Position::Room(*r_type, Room::Outer)) {
                    return true
                }
            },
            [_, Position::Room(r_type, _)] => {
                if *r_type != a {
                    return true
                }
                match self.occupied(Position::Room(*r_type, Room::Inner)) {
                    Some(occupier) if occupier != *r_type => return true, // Wrong amphipod in this room
                    None => return true, // Don't move to Outer if Inner is free
                    _ => (),
                }
            },
            // Don't move between hallway positions
            [Position::Hallway(_), Position::Hallway(_)] => return true,
            _ => (),
        };

        // Check that hallway is clear
        let hallway_range = match [path.start(), path.end()] {
            [Position::Room(a, _), Position::Hallway(h)] => {
                let ah = path.start().hallway_pos();
                if ah < h { ah..=h } else { h..=ah }
            },
            [Position::Hallway(h), Position::Room(a, _)] => {
                let ah = path.end().hallway_pos();
                if ah < h { ah..=h-1 } else { h+1..=ah }
            }
            [p1, p2] => {
                let a = p1.hallway_pos();
                let b = p2.hallway_pos();
                if a < b { a..=b } else { b..=a }
            }
            _ => panic!(),
        };

        for n in hallway_range {
            if self.is_occupied(Position::Hallway(n)) {
                return true
            }
        }

        false
    }

    pub fn positions(&self, a: Amphipod) -> &[Position] {
        &self.0[match a {
            Amphipod::Amber => 0..2,
            Amphipod::Bronze => 2..4,
            Amphipod::Copper => 4..6,
            Amphipod::Desert => 6..8,
        }]
    }

    pub fn positions_mut(&mut self, a: Amphipod) -> &mut [Position] {
        &mut self.0[match a {
            Amphipod::Amber => 0..2,
            Amphipod::Bronze => 2..4,
            Amphipod::Copper => 4..6,
            Amphipod::Desert => 6..8,
        }]
    }

    pub fn normalize(&mut self) {
        self.positions_mut(Amphipod::Amber).sort();
        self.positions_mut(Amphipod::Bronze).sort();
        self.positions_mut(Amphipod::Copper).sort();
        self.positions_mut(Amphipod::Desert).sort();
    }

    pub fn min_energy(&self) -> Energy {
        if *self == GOAL {
            return 0;
        }

        let mut sum = 0;
        for a in [Amphipod::Amber, Amphipod::Bronze, Amphipod::Copper, Amphipod::Desert] {
            let p = self.positions(a);
            let inner = Position::Room(a, Room::Inner);
            let outer = Position::Room(a, Room::Outer);

            sum += cmp::min(
                Path::from([p[0], inner]).cost(a) + Path::from([p[1], outer]).cost(a),
                Path::from([p[1], inner]).cost(a) + Path::from([p[0], outer]).cost(a)
            );
        }
        //println!("min energy: {}", sum);
        sum
    }

    pub fn apply_movement(&mut self, t: StateTransition) {
        let path = t.path;
        let pos = self.positions_mut(t.a);
        match pos.iter_mut().find(|p| **p == path.start()) {
            Some(e) => *e = path.end(),
            None => panic!("Not a valid movement"),
        };
        pos.sort();
        //println!("-> {:?}", self.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path([Position; 2]);

impl Path {
    pub fn start(&self) -> Position { self.0[0] }
    pub fn end(&self) -> Position { self.0[1] }


    pub fn steps(&self) -> usize {
        if self.0[0] == self.0[1] {
            return 0
        }
        let mut p = self.0.clone();
        p.sort();
        match p {
            [Position::Hallway(s), Position::Hallway(e)] => (e - s) as usize,
            [Position::Room(s, Room::Inner), Position::Room(e, Room::Inner)] => Self::horiz_steps(s, e) + 4,
            [Position::Room(s, Room::Outer), Position::Room(e, Room::Outer)] => Self::horiz_steps(s, e) + 2,
            [Position::Room(s, _), Position::Room(e, _)] => Self::horiz_steps(s, e) + 3,
            [Position::Room(s, rm), Position::Hallway(n)] => {
                let rm_pos = match s {
                    Amphipod::Amber => 2,
                    Amphipod::Bronze => 4,
                    Amphipod::Copper => 6,
                    Amphipod::Desert => 8,
                };
                let into_room = match rm {
                    Room::Inner => 2,
                    Room::Outer => 1,
                };

                into_room + (if rm_pos > n { rm_pos - n } else { n - rm_pos }) as usize
            },
            [p1, p2] => {
                //println!("?? {:?}->{:?}", p1, p2);
                100
            },
        }
    }

    pub fn cost(&self, a: Amphipod) -> Energy {
        (self.steps() as Energy) * match a {
            Amphipod::Amber => 1,
            Amphipod::Bronze => 10,
            Amphipod::Copper => 100,
            Amphipod::Desert => 1000,
        }
    }

    fn horiz_steps(start: Amphipod, end: Amphipod) -> usize {
        match (start, end) {
            (Amphipod::Amber, Amphipod::Copper) => 4,
            (Amphipod::Amber, Amphipod::Desert) => 6,
            (Amphipod::Bronze, Amphipod::Desert) => 4,
            (s, e) if s == e => 0,
            _ => 2
        }
    }
}
impl From<[Position; 2]> for Path {
    fn from(a: [Position; 2]) -> Path { Path(a) }
}
impl AsRef<[Position; 2]> for Path {
    fn as_ref(&self) -> &[Position; 2] { &self.0 }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct StateGraph;

impl visit::IntoEdges for StateGraph {
    type Edges = StateTransitions;
    fn edges(self, state: State) -> Self::Edges {
        //println!(": {:?}", &state.0);
        // Generate all possible state transitions
        let mut transitions = Vec::with_capacity(8);
        for (idx, &start) in state.0.iter().enumerate() {
            let a = match idx / 2 {
                0 => Amphipod::Amber,
                1 => Amphipod::Bronze,
                2 => Amphipod::Copper,
                3 => Amphipod::Desert,
                _ => panic!(),
            };

            match start {
                // Don't move if already in room inner
                Position::Room(rm, Room::Inner) if a == rm => { continue; },
                // Don't move if already in full room
                Position::Room(rm, d) if a == rm => {
                    let other = match d {
                        Room::Inner => Room::Outer,
                        Room::Outer => Room::Inner
                    };
                    match state.occupied(Position::Room(rm, other)) {
                        Some(partner) if partner == a => { continue; },
                        _ => (),
                    }
                },
                _ => (),
            };

            for &end in &_ALL_POSITIONS {
                let path = Path::from([start, end]);
                if !state.is_blocked(a, &path) {
                    transitions.push(StateTransition { start: state, a, path })
                }
            }
        }
        transitions.into()
    }
}
impl visit::IntoEdgeReferences for StateGraph {
    type EdgeRef = StateTransition;
    type EdgeReferences = std::iter::Empty<Self::EdgeRef>;

    fn edge_references(self) -> Self::EdgeReferences {
        std::iter::empty()
    }
}
impl visit::IntoNeighbors for StateGraph {
    type Neighbors = PossibleNextStates;

    fn neighbors(self, start: Self::NodeId) -> Self::Neighbors {
        vec!().into()
    }
}

impl visit::Visitable for StateGraph {
    type Map = HashSet<State>;

    fn visit_map(&self) -> Self::Map {
        HashSet::new()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl visit::Data for StateGraph {
    type NodeWeight = ();
    type EdgeWeight = Energy;
}
impl visit::GraphBase for StateGraph {
    type EdgeId = ();
    type NodeId = State;
}
impl visit::GraphRef for StateGraph {
}

pub struct StateTransitions {
    transitions: Vec<StateTransition>,
    n: usize,
}
impl From<Vec<StateTransition>> for StateTransitions {
    fn from(transitions: Vec<StateTransition>) -> Self {
        StateTransitions { transitions, n: 0 }
    }
}
/*
impl From<PossibleNextStates> for StateTransitions {
    fn from(states: PossibleNextStates) -> Self {
        StateTransitions { states }
    }
}
impl From<Vec<State>> for StateTransitions {
    fn from(states:  Vec<State>) -> Self {
        StateTransitions { states: states.into() }
    }
}
*/
impl Iterator for StateTransitions {
    type Item = StateTransition;
    fn next(&mut self) -> Option<Self::Item> {
        match self.transitions.get(self.n) {
            Some(&p) => { self.n += 1; Some(p) },
            None => None
        }
    }
}

pub struct PossibleNextStates {
    states: Vec<State>,
}
impl From<Vec<State>> for PossibleNextStates {
    fn from(states:  Vec<State>) -> Self {
        Self { states }
    }
}
impl Iterator for PossibleNextStates {
    type Item = State;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateTransition {
    start: State,
    a: Amphipod,
    path: Path,
}

pub type Energy = u32;

impl StateTransition {
    pub fn cost(&self) -> Energy {
        self.path.cost(self.a)
    }
}

impl visit::EdgeRef for StateTransition {
    type NodeId = State;
    type EdgeId = ();
    type Weight = Energy;

    fn source(&self) -> Self::NodeId { self.start }
    fn target(&self) -> Self::NodeId {
        let mut target = self.start.clone();
        target.apply_movement(*self);
        target
    }

    fn weight(&self) -> &Self::Weight { panic!() }
    fn id(&self) -> Self::EdgeId {}
}

const _ALL_POSITIONS: [Position; 15] = [
    Position::Room(Amphipod::Amber, Room::Inner),
    Position::Room(Amphipod::Bronze, Room::Inner),
    Position::Room(Amphipod::Copper, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Inner),
    Position::Room(Amphipod::Amber, Room::Outer),
    Position::Room(Amphipod::Bronze, Room::Outer),
    Position::Room(Amphipod::Copper, Room::Outer),
    Position::Room(Amphipod::Desert, Room::Outer),
    Position::Hallway(0),
    Position::Hallway(1),
    Position::Hallway(3),
    Position::Hallway(5),
    Position::Hallway(7),
    Position::Hallway(9),
    Position::Hallway(10),
];

// #############
// #...........#
// ###B#C#B#D###
//   #A#D#C#A#
//   #########
const _SAMPLE_INPUT: State = State([
    Position::Room(Amphipod::Amber, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Inner),
    Position::Room(Amphipod::Amber, Room::Outer),
    Position::Room(Amphipod::Copper, Room::Outer),
    Position::Room(Amphipod::Bronze, Room::Outer),
    Position::Room(Amphipod::Copper, Room::Inner),
    Position::Room(Amphipod::Bronze, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Outer),
]);

// #############
// #...........#
// ###B#B#C#D###
//   #D#C#A#A#
//   #########
const _PROBLEM_INPUT: State = State([
    Position::Room(Amphipod::Copper, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Inner),
    Position::Room(Amphipod::Amber, Room::Outer),
    Position::Room(Amphipod::Bronze, Room::Outer),
    Position::Room(Amphipod::Bronze, Room::Inner),
    Position::Room(Amphipod::Copper, Room::Outer),
    Position::Room(Amphipod::Amber, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Outer),
]);
    
const GOAL: State = State([
    Position::Room(Amphipod::Amber, Room::Inner),
    Position::Room(Amphipod::Amber, Room::Outer),
    Position::Room(Amphipod::Bronze, Room::Inner),
    Position::Room(Amphipod::Bronze, Room::Outer),
    Position::Room(Amphipod::Copper, Room::Inner),
    Position::Room(Amphipod::Copper, Room::Outer),
    Position::Room(Amphipod::Desert, Room::Inner),
    Position::Room(Amphipod::Desert, Room::Outer),
]);


fn find_shortest(start: &State) -> Option<(Energy, Vec<State>)> {
    astar(StateGraph::default(), *start, |s| s == GOAL,
        |m| m.cost(),
        |s| s.min_energy())
}

fn main() {
    println!("for SAMPLE");
    match find_shortest(&_SAMPLE_INPUT) {
        Some((cost, states)) => {
            for s in states {
                println!(": {:?}", s.0);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };

    println!("for PROBLEM");
    match find_shortest(&_PROBLEM_INPUT) {
        Some((cost, states)) => {
            for s in states {
                println!("{:?}", s);
            }
            println!("{} energy", cost);
        },
        None => println!("NO SOLUTION"),
    };

}

#[cfg(test)]
mod tests {
    use super::*;

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
}
