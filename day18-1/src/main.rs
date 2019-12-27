use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::io::Read;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
struct Coord {
    x: i32,
    y: i32,
}

impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

const UNIT_COORDS: [Coord; 4] = [
    Coord { x: 0, y: 1 },
    Coord { x: 0, y: -1 },
    Coord { x: 1, y: 0 },
    Coord { x: -1, y: 0 },
];

struct Map {
    start: Coord,
    keys: HashMap<Coord, u8>,
    doors: HashMap<Coord, u8>,
    walls: HashSet<Coord>,
}

#[derive(Debug)]
struct ReachableKey {
    coord: Coord,
    distance: u32,
    door_set: u32,
    key: u8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct SearchNode {
    distance: u32,
    coord: Coord,
    key_set: u32,
}

impl Map {
    fn parse(s: &str) -> Self {
        let mut walls = HashSet::default();
        let mut doors = HashMap::default();
        let mut keys = HashMap::default();
        let mut start = None;
        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord {
                    x: x as i32,
                    y: y as i32,
                };
                match ch {
                    '@' => start = Some(start.xor(Some(coord)).expect("multiple starts")),
                    '#' => {
                        walls.insert(coord);
                    }
                    'a'..='z' => {
                        keys.insert(coord, ch as u8 - 'a' as u8);
                    }
                    'A'..='Z' => {
                        doors.insert(coord, ch as u8 - 'A' as u8);
                    }
                    '.' => (),
                    _ => panic!("unexpeted char"),
                }
            }
        }
        Self {
            walls,
            doors,
            keys,
            start: start.unwrap(),
        }
    }
    fn reachable_keys_from(&self, coord: Coord) -> Vec<ReachableKey> {
        let mut reachable_keys = Vec::new();
        let mut queue = VecDeque::new();
        let mut seen_set = HashSet::new();
        queue.push_back((coord, 0, 0));
        while let Some((coord, distance, door_set)) = queue.pop_front() {
            if seen_set.insert(coord) {
                if let Some(&key) = self.keys.get(&coord) {
                    reachable_keys.push(ReachableKey {
                        coord,
                        key,
                        distance,
                        door_set,
                    });
                }
                let next_distance = distance + 1;
                let next_door_set =
                    door_set | self.doors.get(&coord).map(|&door| (1 << door)).unwrap_or(0);
                queue.extend(UNIT_COORDS.iter().filter_map(|&unit_coord| {
                    let next_coord = coord + unit_coord;
                    if self.walls.contains(&next_coord) {
                        None
                    } else {
                        Some((next_coord, next_distance, next_door_set))
                    }
                }));
            }
        }
        reachable_keys
    }
    fn solve(&self) -> u32 {
        let interesting_coord_keys = self
            .keys
            .keys()
            .chain([self.start].iter())
            .map(|&coord| (coord, self.reachable_keys_from(coord)))
            .collect::<HashMap<_, _>>();
        let mut queue = BinaryHeap::new();
        queue.push(Reverse(SearchNode {
            distance: 0,
            coord: self.start,
            key_set: 0,
        }));
        let mut seen_set = HashSet::new();
        let all_keys = self
            .keys
            .values()
            .fold(0, |acc, &key| acc | (1 as u32) << key);
        while let Some(Reverse(SearchNode {
            distance,
            coord,
            key_set,
        })) = queue.pop()
        {
            if key_set == all_keys {
                return distance;
            }
            if seen_set.insert((coord, key_set)) {
                let next_nodes = interesting_coord_keys[&coord]
                    .iter()
                    .filter(|ReachableKey { key, .. }| key_set & ((1 as u32) << key) == 0)
                    .filter(|ReachableKey { door_set, .. }| door_set & !key_set == 0)
                    .map(|key| {
                        Reverse(SearchNode {
                            distance: distance + key.distance,
                            coord: key.coord,
                            key_set: key_set | (1 << key.key),
                        })
                    });
                queue.extend(next_nodes);
            }
        }
        panic!("no path");
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let map = Map::parse(&input_string);
    let distance = map.solve();
    println!("{}", distance);
}
