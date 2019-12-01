use std::io::BufRead;

fn fuel_required(mass: u32) -> u32 {
    (mass / 3) - 2
}

fn main() {
    let total_fuel = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<u32>().unwrap())
        .map(fuel_required)
        .sum::<u32>();
    println!("{}", total_fuel);
}
