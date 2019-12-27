use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::Read;

type Coord = (usize, usize);

struct Map {
    floor: HashSet<Coord>,
    portals: HashMap<Coord, Coord>,
    start: Coord,
    goal: Coord,
}

const UNIT_COORDS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

impl Map {
    fn parse(s: &str) -> Self {
        let grid = s
            .lines()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut floor = HashSet::new();
        let mut portals = HashMap::new();
        let mut first_portal_by_name = HashMap::new();
        for (y, row) in grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                match ch {
                    '.' => {
                        floor.insert((x, y));
                        for &(dx, dy) in &UNIT_COORDS {
                            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                            let nch = grid[ny as usize][nx as usize];
                            if nch.is_alphabetic() {
                                let (nnx, nny) = (nx + dx, ny + dy);
                                let nnch = grid[nny as usize][nnx as usize];
                                let name = if nnch < nch {
                                    format!("{}{}", nnch, nch)
                                } else {
                                    format!("{}{}", nch, nnch)
                                };
                                if let Some(existing) = first_portal_by_name.insert(name, (x, y)) {
                                    portals.insert((x, y), existing);
                                    portals.insert(existing, (x, y));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        let start = first_portal_by_name["AA"];
        let goal = first_portal_by_name["ZZ"];
        Self {
            floor,
            portals,
            start,
            goal,
        }
    }
    fn solve(&self) -> u32 {
        let mut queue = BinaryHeap::new();
        let mut seen_set = HashSet::new();
        queue.push(Reverse((0, self.start)));
        while let Some(Reverse((distance, coord))) = queue.pop() {
            if coord == self.goal {
                return distance;
            }
            if seen_set.insert(coord) {
                if let Some(&other_portal) = self.portals.get(&coord) {
                    queue.push(Reverse((distance + 1, other_portal)));
                }
                queue.extend(
                    UNIT_COORDS
                        .iter()
                        .map(|&(dx, dy)| {
                            (
                                (coord.0 as i32 + dx) as usize,
                                (coord.1 as i32 + dy) as usize,
                            )
                        })
                        .filter(|coord| self.floor.contains(&coord))
                        .map(|coord| Reverse((distance + 1, coord))),
                );
            }
        }
        panic!("no path")
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let map = Map::parse(&input_string);
    let distance = map.solve();
    println!("{}", distance);
}
