use std::io::Read;

fn pattern(n: usize) -> impl Iterator<Item = i32> {
    use std::iter::repeat;
    let n = n + 1;
    repeat(0)
        .take(n)
        .chain(repeat(1).take(n))
        .chain(repeat(0).take(n))
        .chain(repeat(-1).take(n))
        .cycle()
        .skip(1)
}

fn process_digit(input: &[i32], n: usize) -> i32 {
    input
        .iter()
        .zip(pattern(n))
        .map(|(&i, p)| i * p)
        .sum::<i32>()
        .abs()
        % 10
}

fn process(input: &[i32]) -> Vec<i32> {
    (0..(input.len()))
        .map(|n| process_digit(input, n))
        .collect()
}

fn process_phases(mut input: Vec<i32>, num_phases: usize) -> Vec<i32> {
    for _ in 0..num_phases {
        input = process(&input);
    }
    input
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let digits = input_string
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect::<Vec<_>>();
    let output = process_phases(digits, 100);
    let output_head = output
        .iter()
        .take(8)
        .map(|i| format!("{}", i))
        .collect::<String>();
    println!("{}", output_head);
}
