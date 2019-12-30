use std::collections::HashSet;
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

fn step(state: u32) -> u32 {
    let mut next_state = 0u32;
    for i in 0..NUM_CELLS {
        let alive = state & (1 << i) != 0;
        let alive_neighbour_count = (i % ROW_SIZE != 0 && state & (1 << (i - 1)) != 0) as u8
            + (i % ROW_SIZE != (ROW_SIZE - 1) && state & (1 << (i + 1)) != 0) as u8
            + (i / ROW_SIZE != 0 && state & (1 << (i - ROW_SIZE)) != 0) as u8
            + (i / ROW_SIZE != (ROW_SIZE - 1) && state & (1 << (i + ROW_SIZE)) != 0) as u8;
        let next_alive = if alive {
            alive_neighbour_count == 1
        } else {
            alive_neighbour_count == 1 || alive_neighbour_count == 2
        };
        next_state |= (next_alive as u32) << i;
    }
    next_state
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

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let mut state = parse(&input_string);
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(state) {
            break;
        }
        state = step(state);
    }
    println!("{}", format_state(state));
    println!("{}", state);
}
