use std::io::Read;

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let asteroid_grid = AsteroidGrid::from_str(&input_string);
    let asteroid_positions = asteroid_grid.asteroid_positions();
    let location = monitoring_station_location(&asteroid_grid, &asteroid_positions);
    let vaporised_coord = nth_vaporised_coord(location, asteroid_grid, 199);
    println!("{}", vaporised_coord.x * 100 + vaporised_coord.y);
}

fn nth_vaporised_coord(
    monitoring_station_location: Coord,
    mut asteroid_grid: AsteroidGrid,
    n: usize,
) -> Coord {
    let asteroid_positions = asteroid_grid.asteroid_positions();
    let mut vaporised = visible_asteroids(
        monitoring_station_location,
        &asteroid_grid,
        &asteroid_positions,
    )
    .collect::<Vec<_>>();
    if vaporised.len() > n {
        vaporised.sort_by(|&a, &b| {
            use std::cmp::Ordering;
            let station_to_a = a - monitoring_station_location;
            let station_to_b = b - monitoring_station_location;
            let angle_a = station_to_a.angle_relative_to_vertical();
            let angle_b = station_to_b.angle_relative_to_vertical();
            if angle_a < angle_b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        vaporised[n]
    } else {
        let next_n = n - vaporised.len();
        for v in vaporised {
            asteroid_grid.remove_asteroid_at(v);
        }
        nth_vaporised_coord(monitoring_station_location, asteroid_grid, next_n)
    }
}

fn monitoring_station_location(
    asteroid_grid: &AsteroidGrid,
    asteroid_positions: &[Coord],
) -> Coord {
    asteroid_positions
        .iter()
        .max_by_key(|&&coord| count_visible_asteroids(coord, asteroid_grid, &asteroid_positions))
        .unwrap()
        .clone()
}

fn visible_asteroids<'a>(
    eye: Coord,
    asteroid_grid: &'a AsteroidGrid,
    asteroid_positions: &'a [Coord],
) -> impl 'a + Iterator<Item = Coord> {
    asteroid_positions
        .iter()
        .filter(move |&&asteroid_position| is_visible(eye, asteroid_position, asteroid_grid))
        .cloned()
}

fn count_visible_asteroids(
    eye: Coord,
    asteroid_grid: &AsteroidGrid,
    asteroid_positions: &[Coord],
) -> usize {
    visible_asteroids(eye, asteroid_grid, asteroid_positions).count()
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
    fn angle_relative_to_vertical(self) -> f64 {
        let x = self.x as f64;
        let y = self.y as f64;
        x.atan2(y)
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
    fn remove_asteroid_at(&mut self, Coord { x, y }: Coord) {
        let index = y as usize * self.width + x as usize;
        self.cells[index] = false;
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
