use std::collections::HashMap;
use std::io::Read;

#[derive(Debug)]
struct Chemical {
    name: String,
    quantity: u32,
}

impl std::str::FromStr for Chemical {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split(' ');
        let quantity = iter.next().ok_or(())?.parse::<u32>().map_err(|_| ())?;
        let name = iter.next().ok_or(())?.to_string();
        Ok(Self { name, quantity })
    }
}

#[derive(Debug)]
struct Reaction {
    inputs: Vec<Chemical>,
    output: Chemical,
}

impl std::str::FromStr for Reaction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut outer_iter = s.trim().split("=>");
        let inputs_str = outer_iter.next().ok_or(())?;
        let output_str = outer_iter.next().ok_or(())?;
        let inputs = inputs_str
            .split(',')
            .map(|s| s.parse::<Chemical>().unwrap())
            .collect::<Vec<_>>();
        let output = output_str.parse::<Chemical>()?;
        Ok(Self { inputs, output })
    }
}

#[derive(Debug)]
struct ReactionTable {
    reaction_by_output_name: HashMap<String, Reaction>,
}

impl std::str::FromStr for ReactionTable {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reaction_by_output_name = s
            .lines()
            .map(|line| {
                let reaction = line.parse::<Reaction>().unwrap();
                (reaction.output.name.clone(), reaction)
            })
            .collect::<HashMap<_, _>>();
        Ok(Self {
            reaction_by_output_name,
        })
    }
}

impl ReactionTable {
    fn fill(&self, name: &str, quantity: u32, quantities: &mut HashMap<String, u32>) -> u32 {
        if name == "ORE" {
            return quantity;
        }
        let current_quantity = quantities.entry(name.to_string()).or_insert(0);
        if *current_quantity >= quantity {
            *current_quantity -= quantity;
            return 0;
        }
        let missing_quantity = quantity - *current_quantity;
        let reaction = self.reaction_by_output_name.get(name).unwrap();
        let num_steps = 1 + ((missing_quantity - 1) / reaction.output.quantity);
        let new_quantity = *current_quantity + (reaction.output.quantity * num_steps) - quantity;
        let mut sum = 0;
        for input in reaction.inputs.iter() {
            sum += self.fill(input.name.as_str(), input.quantity * num_steps, quantities);
        }
        quantities.insert(name.to_string(), new_quantity);
        sum
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let reaction_table = input_string.parse::<ReactionTable>().unwrap();
    let num_ore = reaction_table.fill("FUEL", 1, &mut HashMap::new());
    println!("{}", num_ore);
}
