use std::io::Read;

mod opcode {
    pub const ADD: usize = 1;
    pub const MUL: usize = 2;
    pub const END: usize = 99;
}

#[derive(Clone)]
struct Program {
    ip: usize,
    memory: Vec<usize>,
}

impl Program {
    fn new(memory: Vec<usize>) -> Self {
        Self { ip: 0, memory }
    }
    fn add(&mut self) {
        let lhs = self.memory[self.memory[self.ip + 1]];
        let rhs = self.memory[self.memory[self.ip + 2]];
        let dest = self.memory[self.ip + 3];
        self.memory[dest] = lhs + rhs;
        self.ip += 4;
    }
    fn mul(&mut self) {
        let lhs = self.memory[self.memory[self.ip + 1]];
        let rhs = self.memory[self.memory[self.ip + 2]];
        let dest = self.memory[self.ip + 3];
        self.memory[dest] = lhs * rhs;
        self.ip += 4;
    }
    fn run(mut self, noun: usize, verb: usize) -> usize {
        self.memory[1] = noun;
        self.memory[2] = verb;
        loop {
            match self.memory[self.ip] {
                opcode::ADD => self.add(),
                opcode::MUL => self.mul(),
                opcode::END => break,
                _ => panic!(),
            }
        }
        self.memory[0]
    }
}

const TARGET: usize = 19690720;

fn main() {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .unwrap();
    let memory = input_string
        .split(",")
        .map(|s| s.trim().parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let program = Program::new(memory);
    for noun in 0..100 {
        for verb in 0..100 {
            let output = program.clone().run(noun, verb);
            if output == TARGET {
                println!("{}", 100 * noun + verb);
                return;
            }
        }
    }
}
