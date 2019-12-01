use std::io::BufRead;

fn fuel_required(input: u32) -> u32 {
    match (input / 3).saturating_sub(2) {
        0 => 0,
        non_zero => non_zero + fuel_required(non_zero),
    }
}

fn main() {
    let fuel_required = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<u32>().unwrap())
        .map(fuel_required)
        .sum::<u32>();
    println!("{}", fuel_required);
}
