use std::io::Read;

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let digits = input_string
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect::<Vec<_>>();
    let start_offset = input_string.split_at(7).0.parse::<usize>().unwrap();
    let num_digits = digits.len() * 10_000;
    let mut tail = vec![0u8; num_digits - start_offset];
    for (tail_index, index) in (start_offset..num_digits).enumerate() {
        tail[tail_index] = digits[index % digits.len()];
    }
    for _phase in 0..100 {
        for i in 2..=tail.len() {
            let index = tail.len() - i;
            let next_element = tail[index + 1];
            tail[index] = ((tail[index] as u32 + next_element as u32) % 10) as u8;
        }
    }
    println!(
        "{}",
        tail[0..8]
            .iter()
            .map(|d| format!("{}", d))
            .collect::<String>()
    );
}
