use std::cmp;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::BufRead;

use petgraph::{IntoWeightedEdge};
use petgraph::algo;
use petgraph::graph::NodeIndex;
use petgraph::visit;
use petgraph::visit::{GraphBase, IntoNeighbors};
use petgraph::matrix_graph::{NotZero, UnMatrix};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pos(usize, usize);

impl Pos {
    const fn to_index(self, dim: usize) -> GIndex {
        (self.1 * dim + self.0) as GIndex
    }

    fn to_node_index(self, dim: usize) -> NodeIndex<GIndex> {
        NodeIndex::new(self.to_index(dim) as usize)
    }
    
    fn from_index(idx: GIndex, dim: usize) -> Pos {
        Pos(idx as usize % dim, idx as usize / dim)
    }

    fn from_node_index(idx: NodeIndex<GIndex>, dim: usize) -> Pos {
        Pos::from_index(idx.index() as GIndex, dim)
    }
}

type GIndex = u32;
type Risk = u16;
pub type CaveGraph = UnMatrix<Risk, u8, NotZero<u8>, GIndex>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Right(Pos),
    Down(Pos),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BoundEdge(Edge, usize);

impl BoundEdge {
    fn starts_from(p: Pos, dim: usize) -> Vec<BoundEdge> {
        let mut edges = Vec::with_capacity(2);
        if p.0 < dim - 1 {
            edges.push(BoundEdge(Edge::Right(p), dim));
        }
        if p.1 < dim - 1 {
            edges.push(BoundEdge(Edge::Down(p), dim));
        }
        edges
    }
}

impl IntoWeightedEdge<u8> for BoundEdge {
    type NodeId = GIndex;

    fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, u8) {
        let d = self.1;
        match self.0 {
            Edge::Down(p) => (p.to_index(d), Pos(p.0, p.1 + 1).to_index(d), 1),
            Edge::Right(p) => (p.to_index(d), Pos(p.0 + 1, p.1).to_index(d), 1),
        }
    }
}

pub struct Cave {
    graph: CaveGraph,
    dim: usize,
}

impl Cave {
    fn from_lines(lines: impl Iterator<Item = String>) -> Cave {
        let mut lines = lines.peekable();
        let s = lines.peek().unwrap();

        let dim = s.len();

        let mut graph = CaveGraph::with_capacity(dim * dim);

        // Set all possible edges to wieght 1
        let edges = (0..(dim*dim))
            .map(|i| {
                let x = i % dim;
                let y = i / dim;

                BoundEdge::starts_from(Pos(x, y), dim)
            })
            .flatten();

        graph.extend_with_edges(edges);

        // Now set weights of nodes
        for (y, l) in lines.enumerate() {
            for (x, c) in l.chars().enumerate() {
                let weight = c.to_digit(10).unwrap() as Risk;
                *graph.node_weight_mut(Pos(x, y).to_node_index(dim)) = weight;
            }
        }

        Cave { graph, dim }
    }

    fn dim(&self) -> usize {
        self.dim
    }

    pub fn shortest_path(&self) -> Option<(Risk, Vec<<CaveGraph as GraphBase>::NodeId>)> {
        let dim = self.dim();
        let end = Pos(dim - 1, dim - 1).to_node_index(dim);
        algo::astar(
            &self.graph,
            Pos(0, 0).to_node_index(dim),
            |v| v == end,
            |(_, target, _)| *self.graph.node_weight(target),
            |idx| {
                let p = Pos::from_node_index(idx, dim);
                ((dim - p.0) + (dim - p.1)) as Risk
            })
    }
}

impl fmt::Debug for Cave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            for y in 0..self.dim {
                for x in 0..self.dim {
                    let weight = self.graph.node_weight(Pos(x, y).to_node_index(self.dim));
                    write!(f, "{}", weight)?;
                }
                writeln!(f)?;
            }
        })
    }
}

struct ExtendedCave<'a> {
    cave: &'a Cave,
    repeat: usize,
}

impl<'a> ExtendedCave<'a> {
    pub fn from<'data: 'a>(cave: &'data Cave, repeat: usize) -> ExtendedCave<'a> {
        assert!(repeat > 0);
        ExtendedCave { cave, repeat }
    }

    pub fn dim(&self) -> usize {
        self.cave.dim() * self.repeat
    }

    pub fn node_weight(&self, node: GIndex) -> Risk {
        let dim = self.dim();
        let p = Pos::from_index(node, dim);
        let x = p.0;
        let y = p.1;
        let s_dim = self.cave.dim();

        let sx = x % s_dim;
        let sy = y % s_dim;

        let inc = ((x - sx) + (y - sy)) as Risk;
        let w = *self.cave.graph.node_weight(Pos(sx, sy).to_node_index(s_dim));
        (w + inc - 1) % 9 + 1
    }

    pub fn shortest_path(&self) -> Option<(Risk, Vec<GIndex>)> {
        let dim = self.dim();
        let end = Pos(dim - 1, dim - 1).to_index(dim);
        algo::astar(
            &self,
            Pos(0, 0).to_index(dim),
            |v| v == end,
            |(_, target, _)| self.node_weight(target),
            |idx| {
                let p = Pos::from_index(idx, dim);
                ((dim - p.0) + (dim - p.1)) as Risk
            })
    }
}

impl<'a> visit::GraphBase for ExtendedCave<'a> {
    type EdgeId = (GIndex, GIndex);
    type NodeId = GIndex;
}
impl<'a> visit::Data for ExtendedCave<'a> {
    type NodeWeight = Risk;
    type EdgeWeight = u8;
}

impl<'a, 'b> visit::IntoEdges for &'b ExtendedCave<'a>
{
    type Edges = CaveEdges;

    fn edges(self, v: Self::NodeId) -> Self::Edges {
        CaveEdges(v, self.neighbors(v).collect(), 0)
    }
}

struct CaveEdges(GIndex, Vec<GIndex>, usize);
impl Iterator for CaveEdges {
    type Item = (GIndex, GIndex, &'static u8);
    fn next(&mut self) -> Option<Self::Item> {
        if self.2 >= self.1.len() {
            return None;
        }
        let r = (self.0, self.1[self.2], &1);
        self.2 += 1;
        Some(r)
    }
}

impl<'a, 'b> visit::IntoEdgeReferences for &'b ExtendedCave<'a> {
    type EdgeRef = (GIndex, GIndex, &'static u8);
    type EdgeReferences = CaveFakeEdges;
    fn edge_references(self) -> Self::EdgeReferences {
        Default::default()
    }
}

impl<'a, 'b> visit::Visitable for &'b ExtendedCave<'a> {
    type Map = HashSet<GIndex>;

    fn visit_map(&self) -> Self::Map {
        HashSet::new()
    }
    
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

#[derive(Default)]
struct CaveFakeEdges;
impl Iterator for CaveFakeEdges {
    type Item = (GIndex, GIndex, &'static u8);
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, 'b> visit::IntoNeighbors for &'b ExtendedCave<'a>
{
    type Neighbors = CaveNeighbors;

    fn neighbors(self, v: Self::NodeId) -> Self::Neighbors {
        let dim = self.dim();
        let p = Pos::from_index(v, self.dim());
        let mut neighbors = Vec::with_capacity(4);
        if p.0 + 1 < dim {
            neighbors.push(Pos(p.0 + 1, p.1).to_index(dim));
        }
        if p.1 + 1 < dim {
            neighbors.push(Pos(p.0, p.1 + 1).to_index(dim));
        }
        if p.0 > 0 {
            neighbors.push(Pos(p.0 - 1, p.1).to_index(dim));
        }
        if p.1 > 0 {
            neighbors.push(Pos(p.0, p.1 + 1).to_index(dim));
        }
        CaveNeighbors(neighbors, 0)
    }
}

struct CaveNeighbors(Vec<GIndex>, usize);
impl Iterator for CaveNeighbors {
    type Item = GIndex;
    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.len() {
            return None;
        }
        let r = self.0[self.1];
        self.1 += 1;
        Some(r)
    }
}

fn main() {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap());
    let cave = Cave::from_lines(lines);

    let cave = ExtendedCave::from(&cave, 5);

    println!("dimensions: {0}x{0}", cave.dim());

    //println!("{:?}", cave);

    let (cost, _path) = cave.shortest_path().unwrap();
    //println!("shortest path: {} {:?}", cost, _path);
    println!("shortest path: {}", cost);
}
