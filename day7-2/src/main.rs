use std::collections::VecDeque;
use std::io::Read;

#[derive(Debug)]
enum ParamMode {
    Positional,
    Immediate,
}

impl ParamMode {
    fn from_i32(i32: i32) -> Self {
        match i32 {
            0 => Self::Positional,
            1 => Self::Immediate,
            other => panic!("unexpected param mode code: {}", other),
        }
    }
    fn read(&self, param: i32, memory: &[i32]) -> i32 {
        match self {
            Self::Positional => {
                assert!(param >= 0);
                memory[param as usize]
            }
            Self::Immediate => param,
        }
    }
    fn write(&self, param: i32, value: i32, memory: &mut [i32]) {
        match self {
            Self::Positional => {
                assert!(param >= 0);
                memory[param as usize] = value;
            }
            Self::Immediate => panic!("attempted to write in immediate mode"),
        }
    }
}

#[derive(Debug)]
struct ParamModes {
    encoded: i32,
}

impl ParamModes {
    fn nth(&self, n: u32) -> ParamMode {
        ParamMode::from_i32((self.encoded / (10_i32.pow(n) as i32)) % 10)
    }
}

#[derive(Debug)]
enum Opcode {
    Add,
    Multiply,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Halt,
}

impl Opcode {
    fn from_i32(i32: i32) -> Self {
        match i32 {
            1 => Self::Add,
            2 => Self::Multiply,
            3 => Self::Input,
            4 => Self::Output,
            5 => Self::JumpIfTrue,
            6 => Self::JumpIfFalse,
            7 => Self::LessThan,
            8 => Self::Equals,
            99 => Self::Halt,
            other => panic!("unexpected opcode: {}", other),
        }
    }
}

#[derive(Default, Debug)]
struct IoBuffer {
    values: VecDeque<i32>,
}

impl IoBuffer {
    fn read(&mut self) -> Option<i32> {
        self.values.pop_front()
    }
    fn write(&mut self, value: i32) {
        self.values.push_back(value)
    }
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug)]
struct Instruction {
    opcode: Opcode,
    param_modes: ParamModes,
}

enum Step {
    SetIp(usize),
    WaitForInput,
    Halt,
}

impl Instruction {
    fn decode(encoded: i32) -> Self {
        let opcode = Opcode::from_i32(encoded % 100);
        let param_modes = ParamModes {
            encoded: encoded / 100,
        };
        Self {
            opcode,
            param_modes,
        }
    }
    fn step(
        &self,
        memory: &mut [i32],
        ip: usize,
        input_buffer: &mut IoBuffer,
        output_buffer: &mut IoBuffer,
    ) -> Step {
        match self.opcode {
            Opcode::Add => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = lhs + rhs;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Step::SetIp(ip + 4)
            }
            Opcode::Multiply => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = lhs * rhs;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Step::SetIp(ip + 4)
            }
            Opcode::Input => {
                if let Some(value) = input_buffer.read() {
                    let param = memory[ip + 1];
                    self.param_modes.nth(0).write(param, value, memory);
                    Step::SetIp(ip + 2)
                } else {
                    Step::WaitForInput
                }
            }
            Opcode::Output => {
                let param = memory[ip + 1];
                let output = self.param_modes.nth(0).read(param, memory);
                output_buffer.write(output);
                Step::SetIp(ip + 2)
            }
            Opcode::JumpIfTrue => {
                let cond_param = memory[ip + 1];
                let target_param = memory[ip + 2];
                let cond = self.param_modes.nth(0).read(cond_param, memory);
                if cond != 0 {
                    let target = self.param_modes.nth(1).read(target_param, memory);
                    Step::SetIp(target as usize)
                } else {
                    Step::SetIp(ip + 3)
                }
            }
            Opcode::JumpIfFalse => {
                let cond_param = memory[ip + 1];
                let target_param = memory[ip + 2];
                let cond = self.param_modes.nth(0).read(cond_param, memory);
                if cond == 0 {
                    let target = self.param_modes.nth(1).read(target_param, memory);
                    Step::SetIp(target as usize)
                } else {
                    Step::SetIp(ip + 3)
                }
            }
            Opcode::LessThan => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = (lhs < rhs) as i32;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Step::SetIp(ip + 4)
            }
            Opcode::Equals => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = (lhs == rhs) as i32;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Step::SetIp(ip + 4)
            }
            Opcode::Halt => Step::Halt,
        }
    }
}

#[derive(Clone, Copy)]
enum Run {
    Halt,
    WaitForInput { ip: usize },
}

fn run(
    mut ip: usize,
    memory: &mut [i32],
    input_buffer: &mut IoBuffer,
    output_buffer: &mut IoBuffer,
) -> Run {
    loop {
        let instruction = Instruction::decode(memory[ip]);
        match instruction.step(memory, ip, input_buffer, output_buffer) {
            Step::Halt => return Run::Halt,
            Step::SetIp(next_ip) => ip = next_ip,
            Step::WaitForInput => return Run::WaitForInput { ip },
        }
    }
}

fn for_each_permutation_rec<T, F: FnMut(&[T])>(values: &mut [T], prefix_n: usize, f: &mut F) {
    if prefix_n == 1 {
        f(values);
    } else {
        for_each_permutation_rec(values, prefix_n - 1, f);
        for i in 0..(prefix_n - 1) {
            if prefix_n % 2 == 0 {
                values.swap(i, prefix_n - 1);
            } else {
                values.swap(0, prefix_n - 1);
            }
            for_each_permutation_rec(values, prefix_n - 1, f);
        }
    }
}

fn for_each_permutation<T, F: FnMut(&[T])>(values: &mut [T], mut f: F) {
    for_each_permutation_rec(values, values.len(), &mut f);
}

struct Amp {
    memory: Vec<i32>,
    status: Run,
}

impl Amp {
    fn new(mut memory: Vec<i32>, phase: i32) -> Self {
        let mut input_buffer = IoBuffer::default();
        input_buffer.write(phase);
        let mut output_buffer = IoBuffer::default();
        let status = match run(0, &mut memory, &mut input_buffer, &mut output_buffer) {
            Run::Halt => panic!("unexpected halt during init"),
            status @ Run::WaitForInput { .. } => status,
        };
        assert!(input_buffer.is_empty());
        assert!(output_buffer.is_empty());
        Self { memory, status }
    }
    fn status(&self) -> Run {
        self.status
    }
    fn run(&mut self, input_buffer: &mut IoBuffer, output_buffer: &mut IoBuffer) {
        self.status = match self.status {
            Run::Halt => Run::Halt,
            Run::WaitForInput { ip } => run(ip, &mut self.memory, input_buffer, output_buffer),
        };
    }
}

fn run_amps(phases: &[i32], program: &[i32]) -> i32 {
    let mut amps = phases
        .iter()
        .map(|&phase| Amp::new(program.to_vec(), phase))
        .collect::<Vec<_>>();
    let mut input_buffer = IoBuffer::default();
    let mut output_buffer = IoBuffer::default();
    input_buffer.write(0);
    loop {
        for amp in amps.iter_mut() {
            if let Run::Halt = amp.status() {
                return input_buffer.read().expect("no output");
            }
            amp.run(&mut input_buffer, &mut output_buffer);
            assert!(!output_buffer.is_empty());
            std::mem::swap(&mut input_buffer, &mut output_buffer);
        }
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .unwrap();
    let program = input_string
        .split(",")
        .map(|s| s.trim().parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    let mut max_output = 0;
    for_each_permutation(&mut [5, 6, 7, 8, 9], |perm| {
        let output = run_amps(perm, &program);
        max_output = max_output.max(output);
    });
    println!("{}", max_output);
}
