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
    let mut adjacent_count = 0;
    let mut adjacent_pair = false;
    for (right, left) in DigitsReversed(password).zip(DigitsReversed(password / 10)) {
        if right < left {
            return false;
        } else if left == right {
            adjacent_count += 1;
        } else {
            if adjacent_count == 1 {
                adjacent_pair = true;
            }
            adjacent_count = 0;
        }
    }
    return adjacent_pair || adjacent_count == 1;
}

fn main() {
    let num_valid = (183564..657474).filter(|&n| is_valid(n)).count();
    println!("{}", num_valid);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples() {
        assert!(is_valid(111122));
        assert!(is_valid(112222));
        assert!(is_valid(112345));
        assert!(is_valid(123345));
        assert!(is_valid(112233));
        assert!(!is_valid(111111));
        assert!(!is_valid(123444));
    }
}
