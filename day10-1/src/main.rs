use std::io::Read;

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let asteroid_grid = AsteroidGrid::from_str(&input_string);
    let max = best_num_asteroids(&asteroid_grid);
    println!("{}", max);
}

fn best_num_asteroids(asteroid_grid: &AsteroidGrid) -> usize {
    let asteroid_positions = asteroid_grid.asteroid_positions();
    asteroid_positions
        .iter()
        .map(|&coord| count_visible_asteroids(coord, asteroid_grid, &asteroid_positions))
        .max()
        .unwrap()
}

fn count_visible_asteroids(
    eye: Coord,
    asteroid_grid: &AsteroidGrid,
    asteroid_positions: &[Coord],
) -> usize {
    asteroid_positions
        .iter()
        .filter(|&&asteroid_position| is_visible(eye, asteroid_position, asteroid_grid))
        .count()
}

fn is_visible(eye: Coord, mut coord: Coord, asteroid_grid: &AsteroidGrid) -> bool {
    if coord == eye {
        return false;
    }
    let coord_to_eye_step = (eye - coord).lowest_terms();
    coord = coord + coord_to_eye_step;
    while coord != eye {
        if asteroid_grid.has_asteroid_at(coord) {
            return false;
        }
        coord = coord + coord_to_eye_step;
    }
    true
}

fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn lowest_terms(self) -> Self {
        if self.x == 0 {
            Self {
                x: 0,
                y: self.y / self.y.abs(),
            }
        } else if self.y == 0 {
            Self {
                x: self.x / self.x.abs(),
                y: 0,
            }
        } else {
            let divisor = gcd(self.x.abs(), self.y.abs());
            Self {
                x: self.x / divisor,
                y: self.y / divisor,
            }
        }
    }
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

impl std::ops::Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

struct AsteroidGrid {
    cells: Vec<bool>,
    width: usize,
    height: usize,
}

impl AsteroidGrid {
    fn from_str(s: &str) -> Self {
        let mut width = None;
        let mut cells = Vec::new();
        for line in s.lines() {
            width = Some(line.len());
            for ch in line.chars() {
                if ch == '#' {
                    cells.push(true);
                } else {
                    assert_eq!(ch, '.');
                    cells.push(false);
                }
            }
        }
        let width = width.unwrap();
        Self {
            width,
            height: cells.len() / width,
            cells,
        }
    }
    fn has_asteroid_at(&self, Coord { x, y }: Coord) -> bool {
        let index = y as usize * self.width + x as usize;
        self.cells[index]
    }
    fn asteroid_positions(&self) -> Vec<Coord> {
        let mut ret = Vec::new();
        for y in 0..(self.height as i32) {
            for x in 0..(self.width as i32) {
                let coord = Coord { x, y };
                if self.has_asteroid_at(coord) {
                    ret.push(coord);
                }
            }
        }
        ret
    }
}
