use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::Read;

type Coord = (usize, usize);

struct Map {
    floor: HashSet<Coord>,
    portals: HashMap<Coord, (Coord, i8)>,
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
        let width = grid[0].len();
        let height = grid.len();
        let mut floor = HashSet::new();
        let mut portals = HashMap::new();
        let mut first_portal_by_name = HashMap::new();
        for (y, row) in grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                match ch {
                    '.' => {
                        floor.insert((x, y));
                        let side = if x == 2 || y == 2 || x == width - 3 || y == height - 3 {
                            -1i8
                        } else {
                            1i8
                        };
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
                                if let Some((existing, existing_side)) =
                                    first_portal_by_name.insert(name, ((x, y), side))
                                {
                                    portals.insert((x, y), (existing, side));
                                    portals.insert(existing, ((x, y), existing_side));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        let start = first_portal_by_name["AA"].0;
        let goal = first_portal_by_name["ZZ"].0;
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
        queue.push(Reverse((0, self.start, 0)));
        while let Some(Reverse((distance, coord, depth))) = queue.pop() {
            if coord == self.goal && depth == 0 {
                return distance;
            }
            if seen_set.insert((coord, depth)) {
                if let Some(&(other_portal, side)) = self.portals.get(&coord) {
                    if side == 1 || depth > 0 {
                        let next_depth = (depth as i32 + side as i32) as usize;
                        queue.push(Reverse((distance + 1, other_portal, next_depth)));
                    }
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
                        .map(|coord| Reverse((distance + 1, coord, depth))),
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
