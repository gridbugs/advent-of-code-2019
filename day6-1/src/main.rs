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
    fn count_orbits(&self, root: &NodeName) -> Count {
        if let Some(children) = self.edges.get(root) {
            let child_counts = children
                .iter()
                .map(|node_name| self.count_orbits(node_name))
                .sum::<Count>();
            Count {
                num_nodes: child_counts.num_nodes + 1,
                num_orbits: child_counts.num_orbits + child_counts.num_nodes,
            }
        } else {
            Count {
                num_nodes: 1,
                num_orbits: 0,
            }
        }
    }
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
    let orbit_count = directed_graph.count_orbits(&"COM".to_string());
    println!("{}", orbit_count.num_orbits);
}
