use std::io::BufRead;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn unit(self) -> (i32, i32) {
        match self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Up => (0, -1),
            Self::Down => (0, 1),
        }
    }
}

#[derive(Debug)]
struct Step {
    direction: Direction,
    distance: usize,
}

impl std::str::FromStr for Step {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction_indicator = s.chars().next().ok_or(())?;
        let (_, distance_str) = s.split_at(direction_indicator.len_utf8());
        let direction = match direction_indicator {
            'L' => Direction::Left,
            'R' => Direction::Right,
            'U' => Direction::Up,
            'D' => Direction::Down,
            _ => return Err(()),
        };
        let distance = distance_str.parse().map_err(|_| ())?;
        Ok(Self {
            direction,
            distance,
        })
    }
}

impl Step {
    fn units(&self) -> impl Iterator<Item = (i32, i32)> {
        std::iter::repeat(self.direction)
            .take(self.distance)
            .map(|direction| direction.unit())
    }
}

fn all_visited(steps: &[Step]) -> impl '_ + Iterator<Item = (i32, i32)> {
    steps
        .iter()
        .flat_map(|step| step.units())
        .scan((0, 0), |(x, y), (dx, dy)| {
            *x += dx;
            *y += dy;
            Some((*x, *y))
        })
}

fn distance_to_intersection(a: &[Step], b: &[Step]) -> Option<i32> {
    let a_visited = all_visited(a).collect::<std::collections::HashSet<_>>();
    let b_visited = all_visited(b).collect::<std::collections::HashSet<_>>();
    a_visited
        .intersection(&b_visited)
        .map(|(x, y)| x.abs() + y.abs())
        .min()
}

fn main() {
    let wires = std::io::stdin()
        .lock()
        .lines()
        .map(|s| {
            s.unwrap()
                .trim()
                .split(',')
                .map(|s| s.parse::<Step>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let distance = distance_to_intersection(&wires[0], &wires[1]).unwrap();
    println!("{}", distance);
}
