use std::collections::HashMap;
use std::io::Read;

#[derive(Debug)]
struct Chemical {
    name: String,
    quantity: u64,
}

impl std::str::FromStr for Chemical {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split(' ');
        let quantity = iter.next().ok_or(())?.parse::<u64>().map_err(|_| ())?;
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
    fn fill(&self, name: &str, quantity: u64, quantities: &mut HashMap<String, u64>) -> u64 {
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
    fn find_max_fuel(&self, target_num_ore: u64, larger_than_max_fuel_guess_exp: u64) -> u64 {
        let larger_than_max_fuel_guess = 1 << larger_than_max_fuel_guess_exp;
        let base_num_ore = self.fill("FUEL", larger_than_max_fuel_guess, &mut HashMap::new());
        assert!(base_num_ore > target_num_ore);
        // find maximum fuel quantity s.t. the amount of ore is <= ore_amount
        let mut fuel = larger_than_max_fuel_guess;
        for i in 0..larger_than_max_fuel_guess_exp {
            let exp = larger_than_max_fuel_guess_exp - i - 1;
            let num_ore = self.fill("FUEL", fuel, &mut HashMap::new());
            if num_ore > target_num_ore {
                fuel -= 1 << exp;
            } else if num_ore < target_num_ore {
                fuel += 1 << exp;
            } else {
                break;
            }
        }
        fuel
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin().read_to_string(&mut input_string).unwrap();
    let reaction_table = input_string.parse::<ReactionTable>().unwrap();
    let max_fuel = reaction_table.find_max_fuel(1_000_000_000_000, 30);
    println!("{}", max_fuel);
}
