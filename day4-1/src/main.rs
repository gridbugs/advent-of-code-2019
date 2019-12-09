struct DigitsReversed(u32);
impl Iterator for DigitsReversed {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let digit = self.0 % 10;
        self.0 /= 10;
        Some(digit)
    }
}

fn is_valid(password: u32) -> bool {
    if password < 100000 || password >= 1000000 {
        return false;
    }
    let mut adjacent = false;
    for (right, left) in DigitsReversed(password).zip(DigitsReversed(password / 10)) {
        if right < left {
            return false;
        } else {
            adjacent = adjacent || left == right;
        }
    }
    return adjacent;
}

fn main() {
    let num_valid = (183564..657474).filter(|&n| is_valid(n)).count();
    println!("{}", num_valid);
}
