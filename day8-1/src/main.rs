use std::io::Read;

const WIDTH: usize = 25;
const HEIGHT: usize = 6;
const SIZE: usize = WIDTH * HEIGHT;

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let digits = input_string
        .trim()
        .chars()
        .map(|c| {
            let mut buf = [0; 4];
            c.encode_utf8(&mut buf).parse::<u8>().unwrap()
        })
        .collect::<Vec<_>>();
    let layer = layer_with_most_zeros(&digits, SIZE);
    let num_1s = count_values(layer, 1);
    let num_2s = count_values(layer, 2);
    println!("{}", num_1s * num_2s);
}

fn count_values(layer: &[u8], value: u8) -> usize {
    layer.iter().filter(|&&digit| digit == value).count()
}

fn layer_with_most_zeros(data: &[u8], layer_size: usize) -> &[u8] {
    data.chunks_exact(layer_size)
        .min_by_key(|layer| count_values(layer, 0))
        .unwrap()
}
