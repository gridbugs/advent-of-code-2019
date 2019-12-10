use std::collections::HashMap;
use std::io::BufRead;

type NodeName = String;

#[derive(Default)]
struct DirectedGraph {
    edges: HashMap<NodeName, Vec<NodeName>>,
}

#[derive(Default)]
struct Count {
    num_nodes: usize,
    num_orbits: usize,
}

impl std::iter::Sum for Count {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Count::default(), |count, acc| Count {
            num_nodes: count.num_nodes + acc.num_nodes,
            num_orbits: count.num_orbits + acc.num_orbits,
        })
    }
}

impl DirectedGraph {
    fn connect(&mut self, Orbit { parent, child }: Orbit) {
        self.edges
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }
    fn distance(&self, root: &NodeName) -> Option<Distance> {
        if root == "YOU" {
            return Some(Distance::You(0));
        }
        if root == "SAN" {
            return Some(Distance::Santa(0));
        }
        if let Some(children) = self.edges.get(root) {
            let mut you = None;
            let mut santa = None;
            for child in children {
                if let Some(distance) = self.distance(child) {
                    match distance {
                        both @ Distance::Both(_) => return Some(both),
                        Distance::Santa(d) => santa = Some(d),
                        Distance::You(d) => you = Some(d),
                    }
                }
            }
            match (you, santa) {
                (Some(you), Some(santa)) => Some(Distance::Both(you + santa)),
                (Some(you), None) => Some(Distance::You(you + 1)),
                (None, Some(santa)) => Some(Distance::Santa(santa + 1)),
                (None, None) => None,
            }
        } else {
            None
        }
    }
}

enum Distance {
    You(usize),
    Santa(usize),
    Both(usize),
}

struct Orbit {
    parent: NodeName,
    child: NodeName,
}

impl std::str::FromStr for Orbit {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(")");
        let parent = split.next().ok_or(())?.to_string();
        let child = split.next().ok_or(())?.to_string();
        Ok(Self { parent, child })
    }
}

fn main() {
    let orbits = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<Orbit>().unwrap())
        .collect::<Vec<_>>();
    let mut directed_graph = DirectedGraph::default();
    for orbit in orbits {
        directed_graph.connect(orbit);
    }
    if let Some(Distance::Both(distance)) = directed_graph.distance(&"COM".to_string()) {
        println!("{}", distance);
    }
}
