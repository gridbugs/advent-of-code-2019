use std::io::BufRead;

#[derive(Clone, Copy)]
enum Shuffle {
    DealWithIncrement(u32),
    Cut(i32),
    DealOntoNewStack,
}

impl std::str::FromStr for Shuffle {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "deal into new stack" {
            return Ok(Self::DealOntoNewStack);
        }
        let words = s.split_whitespace().collect::<Vec<_>>();
        let prefix = words[0..(words.len() - 1)].join(" ");
        let end = words.last().unwrap();
        match prefix.as_str() {
            "cut" => Ok(Self::Cut(end.parse().unwrap())),
            "deal with increment" => Ok(Self::DealWithIncrement(end.parse().unwrap())),
            _ => Err(()),
        }
    }
}

fn normalise(x: i128) -> i128 {
    let mut normalised = x % DECK_SIZE as i128;
    if normalised < 0 {
        normalised += DECK_SIZE as i128;
    }
    normalised
}

fn mod_inverse(n: i128) -> i128 {
    let Egcd { coef_a, .. } = egcd(n, DECK_SIZE as i128);
    normalise(coef_a)
}

impl Shuffle {
    fn update_linear_equation(self, LinearEquation { mul, add }: LinearEquation) -> LinearEquation {
        match self {
            Self::DealOntoNewStack => LinearEquation {
                mul: normalise(-mul),
                add: normalise(-add - 1),
            },
            Self::Cut(n) => LinearEquation {
                mul,
                add: normalise(add - n as i128),
            },
            Self::DealWithIncrement(n) => LinearEquation {
                mul: normalise(mul * n as i128),
                add: normalise(add * n as i128),
            },
        }
    }
    fn update_reverse_linear_equation(
        self,
        LinearEquation { mul, add }: LinearEquation,
    ) -> LinearEquation {
        match self {
            Self::DealOntoNewStack => LinearEquation {
                mul: normalise(-mul),
                add: normalise(-add - 1),
            },
            Self::Cut(n) => LinearEquation {
                mul,
                add: normalise(add + n as i128),
            },
            Self::DealWithIncrement(n) => {
                let mod_inverse = mod_inverse(n as i128);
                LinearEquation {
                    mul: normalise(mul * mod_inverse),
                    add: normalise(add * mod_inverse),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct LinearEquation {
    mul: i128,
    add: i128,
}

impl Default for LinearEquation {
    fn default() -> Self {
        Self { mul: 1, add: 0 }
    }
}

impl LinearEquation {
    fn calculate(&self, x: i128) -> i128 {
        (((self.mul * x + self.add) % DECK_SIZE as i128) + DECK_SIZE as i128) % DECK_SIZE as i128
    }
    fn calculate_repeated(self, x: i128, num: u64) -> i128 {
        let mut equation_2_exp_i = self;
        let mut running_equation = Self::default();
        for i in 0..64 {
            if num & (1 << i) != 0 {
                running_equation = running_equation.nest(equation_2_exp_i);
            }
            equation_2_exp_i = equation_2_exp_i.nest(equation_2_exp_i);
        }
        running_equation.calculate(x)
    }
    fn nest(self, other: Self) -> Self {
        let mul = normalise(self.mul * other.mul);
        let add = normalise(self.mul * other.add + self.add);
        Self { mul, add }
    }
}

const DECK_SIZE: usize = 119315717514047;
const NUM_REPETITIONS: u64 = 101741582076661;

#[derive(Debug)]
struct Egcd {
    gcd: i128,
    coef_a: i128,
    coef_b: i128,
}

fn egcd(a: i128, b: i128) -> Egcd {
    let mut s = 0;
    let mut old_s = 1;
    let mut t = 1;
    let mut old_t = 0;
    let mut r = b;
    let mut old_r = a;
    while r != 0 {
        let q = old_r / r;
        let new_r = old_r - (q * r);
        old_r = r;
        r = new_r;
        let new_s = old_s - (q * s);
        old_s = s;
        s = new_s;
        let new_t = old_t - (q * t);
        old_t = t;
        t = new_t;
    }
    Egcd {
        gcd: old_r,
        coef_a: old_s,
        coef_b: old_t,
    }
}

fn main() {
    let shuffles = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<Shuffle>().unwrap())
        .collect::<Vec<_>>();
    let reverse_linear_equation = shuffles
        .iter()
        .rev()
        .fold(LinearEquation::default(), |acc, shuffle| {
            shuffle.update_reverse_linear_equation(acc)
        });
    let solution = reverse_linear_equation.calculate_repeated(2020, NUM_REPETITIONS);
    println!("{}", solution);
}
