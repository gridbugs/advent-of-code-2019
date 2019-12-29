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

impl Shuffle {
    fn run(self, deck: &mut DoubleBufferedDeck) {
        match self {
            Self::DealOntoNewStack => deck.deal_onto_new_stack(),
            Self::Cut(n) => deck.cut(n),
            Self::DealWithIncrement(n) => deck.deal_with_increment(n),
        }
    }
}

const DECK_SIZE: usize = 10007;

struct DoubleBufferedDeck {
    deck: Vec<usize>,
    other: Vec<usize>,
}

impl DoubleBufferedDeck {
    fn new(size: usize) -> Self {
        let deck = (0..size).collect::<Vec<_>>();
        Self {
            other: deck.clone(),
            deck,
        }
    }
    fn deal_onto_new_stack(&mut self) {
        self.deck.reverse();
    }
    fn cut(&mut self, n: i32) {
        let split_index = if n > 0 {
            n as usize
        } else {
            self.deck.len() - (n.abs() as usize)
        };
        let start = &self.deck[split_index..];
        let end = &self.deck[..split_index];
        &mut self.other[..start.len()].copy_from_slice(start);
        &mut self.other[start.len()..].copy_from_slice(end);
        std::mem::swap(&mut self.deck, &mut self.other);
    }
    fn deal_with_increment(&mut self, n: u32) {
        let mut index = 0;
        for &card in &self.deck {
            self.other[index] = card;
            index = (index + n as usize) % self.deck.len();
        }
        std::mem::swap(&mut self.deck, &mut self.other);
    }
    fn card_position(&self, search_card: usize) -> usize {
        for (index, &current_card) in self.deck.iter().enumerate() {
            if current_card == search_card {
                return index;
            }
        }
        panic!("card not found");
    }
}

fn main() {
    let mut deck = DoubleBufferedDeck::new(DECK_SIZE);
    for shuffle in std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<Shuffle>().unwrap())
    {
        shuffle.run(&mut deck);
    }
    println!("{:?}", deck.card_position(2019));
}
