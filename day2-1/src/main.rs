use std::io::Read;

mod opcode {
    pub const ADD: usize = 1;
    pub const MUL: usize = 2;
    pub const END: usize = 99;
}

fn add(pc: usize, program: &mut [usize]) {
    let lhs = program[program[pc + 1]];
    let rhs = program[program[pc + 2]];
    program[program[pc + 3]] = lhs + rhs;
}
fn mul(pc: usize, program: &mut [usize]) {
    let lhs = program[program[pc + 1]];
    let rhs = program[program[pc + 2]];
    program[program[pc + 3]] = lhs * rhs;
}

fn run_program(program: &mut [usize]) {
    let mut pc = 0;
    loop {
        match program[pc] {
            opcode::ADD => add(pc, program),
            opcode::MUL => mul(pc, program),
            opcode::END => break,
            _ => panic!(),
        }
        pc += 4;
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .unwrap();
    let mut program = input_string
        .split(",")
        .map(|s| s.trim().parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    program[1] = 12;
    program[2] = 2;
    run_program(&mut program);
    println!("{}", program[0]);
}
