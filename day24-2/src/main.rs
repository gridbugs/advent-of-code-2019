use std::io::Read;

const NUM_CELLS: u8 = 25;
const ROW_SIZE: u8 = 5;

fn parse(s: &str) -> u32 {
    s.chars()
        .filter(|&c| c != '\n')
        .enumerate()
        .map(|(i, ch)| if ch == '#' { 1 << i } else { 0 })
        .sum()
}

#[derive(Clone, Copy, Default)]
struct Level {
    cells: u32,
}

const INNER_TOP_IDX: u8 = 7;
const INNER_BOTTOM_IDX: u8 = 17;
const INNER_LEFT_IDX: u8 = 11;
const INNER_RIGHT_IDX: u8 = 13;
const MID_IDX: u8 = 12;

impl Level {
    fn outer_count_top(self) -> u8 {
        (0..ROW_SIZE)
            .map(|i| (self.cells & (1 << i) != 0) as u8)
            .sum()
    }
    fn outer_count_bottom(self) -> u8 {
        (0..ROW_SIZE)
            .map(|i| (self.cells & (1 << (i + (NUM_CELLS - ROW_SIZE))) != 0) as u8)
            .sum()
    }
    fn outer_count_left(self) -> u8 {
        (0..ROW_SIZE)
            .map(|i| (self.cells & (1 << (i * ROW_SIZE)) != 0) as u8)
            .sum()
    }
    fn outer_count_right(self) -> u8 {
        (0..ROW_SIZE)
            .map(|i| (self.cells & (1 << (i * ROW_SIZE + ROW_SIZE - 1)) != 0) as u8)
            .sum()
    }
    fn count(self) -> u64 {
        (0..NUM_CELLS)
            .map(|i| (self.cells & (1 << i) != 0) as u64)
            .sum()
    }
}

struct State {
    levels: Vec<Level>,
}

struct DoubleBufferedState {
    current: State,
    next: State,
}

impl DoubleBufferedState {
    fn new(start_level_cells: u32, num_steps: usize) -> Self {
        let current = State::new(start_level_cells, num_steps);
        let next = State::new(0, num_steps);
        Self { current, next }
    }
    fn step(&mut self, minute: usize) {
        self.current.step(minute, &mut self.next);
        std::mem::swap(&mut self.current, &mut self.next);
    }
}

impl State {
    fn new(start_level_cells: u32, num_steps: usize) -> Self {
        let num_levels = (num_steps + 1) * 2 + 1;
        let mut levels = Vec::new();
        levels.resize_with(num_levels, Level::default);
        levels[num_steps + 1].cells = start_level_cells;
        Self { levels }
    }
    fn count(&self) -> u64 {
        self.levels.iter().map(|l| l.count()).sum()
    }
    fn step(&self, minute: usize, output: &mut Self) {
        let minute = minute + 1;
        let range = (self.levels.len() / 2) - minute..=(self.levels.len() / 2) + minute;
        for (level_i_offset, output_level) in output.levels[range].iter_mut().enumerate() {
            let level_i = (self.levels.len() / 2) + level_i_offset - minute;
            let this_level = self.levels[level_i];
            let inner_level = self.levels[level_i + 1];
            let outer_level = self.levels[level_i - 1];
            output_level.cells = 0;
            for cell_i in 0..NUM_CELLS {
                if cell_i == MID_IDX {
                    continue;
                };
                let alive = this_level.cells & (1 << cell_i) != 0;
                let mut alive_neighbour_count = if cell_i % ROW_SIZE == 0 {
                    (outer_level.cells & (1 << INNER_LEFT_IDX) != 0) as u8
                } else {
                    (this_level.cells & (1 << (cell_i - 1)) != 0) as u8
                } + if cell_i % ROW_SIZE == ROW_SIZE - 1 {
                    (outer_level.cells & (1 << INNER_RIGHT_IDX) != 0) as u8
                } else {
                    (this_level.cells & (1 << (cell_i + 1)) != 0) as u8
                } + if cell_i / ROW_SIZE == 0 {
                    (outer_level.cells & (1 << INNER_TOP_IDX) != 0) as u8
                } else {
                    (this_level.cells & (1 << (cell_i - ROW_SIZE)) != 0) as u8
                } + if cell_i / ROW_SIZE == ROW_SIZE - 1 {
                    (outer_level.cells & (1 << INNER_BOTTOM_IDX) != 0) as u8
                } else {
                    (this_level.cells & (1 << (cell_i + ROW_SIZE)) != 0) as u8
                };
                alive_neighbour_count += match cell_i {
                    INNER_TOP_IDX => inner_level.outer_count_top(),
                    INNER_BOTTOM_IDX => inner_level.outer_count_bottom(),
                    INNER_LEFT_IDX => inner_level.outer_count_left(),
                    INNER_RIGHT_IDX => inner_level.outer_count_right(),
                    _ => 0,
                };
                let next_alive = if alive {
                    alive_neighbour_count == 1
                } else {
                    alive_neighbour_count == 1 || alive_neighbour_count == 2
                };
                output_level.cells |= (next_alive as u32) << cell_i;
            }
        }
    }
}

fn format_state(state: u32) -> String {
    (0..NUM_CELLS)
        .map(|i| {
            let ch = if state & (1 << i) != 0 { '#' } else { '.' };
            if i % ROW_SIZE == (ROW_SIZE - 1) {
                format!("{}\n", ch)
            } else {
                format!("{}", ch)
            }
        })
        .collect()
}

const NUM_STEPS: usize = 200;

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let initial_mid_level = parse(&input_string);
    let mut state = DoubleBufferedState::new(initial_mid_level, NUM_STEPS);
    for i in 0..NUM_STEPS {
        state.step(i);
    }
    println!("{}", state.current.count());
}
