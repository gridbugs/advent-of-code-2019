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
    let rendered = render(&digits, SIZE);
    display(&rendered, WIDTH, HEIGHT);
}

#[derive(Clone, Copy)]
enum Colour {
    White,
    Black,
    Transparent,
}

impl Colour {
    fn from_u8(u8: u8) -> Self {
        match u8 {
            0 => Self::Black,
            1 => Self::White,
            2 => Self::Transparent,
            _ => panic!(),
        }
    }
    fn blend(self, other: Self) -> Self {
        if let Colour::Transparent = self {
            other
        } else {
            self
        }
    }
}

fn render(data: &[u8], layer_size: usize) -> Vec<Colour> {
    let mut output = vec![Colour::Transparent; layer_size];
    for layer in data.chunks_exact(layer_size) {
        for (colour, output) in layer
            .iter()
            .cloned()
            .map(Colour::from_u8)
            .zip(output.iter_mut())
        {
            *output = (*output).blend(colour);
        }
    }
    output
}

fn display(rendered: &[Colour], width: usize, _height: usize) {
    for row in rendered.chunks_exact(width) {
        for colour in row {
            let ch = match colour {
                Colour::White => '#',
                Colour::Black => '.',
                Colour::Transparent => ' ',
            };
            print!("{}", ch);
        }
        println!("");
    }
}
