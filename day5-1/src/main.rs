use std::collections::VecDeque;
use std::io::Read;

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

struct ParamModes {
    encoded: i32,
}

impl ParamModes {
    fn nth(&self, n: u32) -> ParamMode {
        ParamMode::from_i32((self.encoded / (10_i32.pow(n) as i32)) % 10)
    }
}

enum Opcode {
    Add,
    Multiply,
    Input,
    Output,
    Halt,
}

impl Opcode {
    fn from_i32(i32: i32) -> Self {
        match i32 {
            1 => Self::Add,
            2 => Self::Multiply,
            3 => Self::Input,
            4 => Self::Output,
            99 => Self::Halt,
            other => panic!("unexpectde opcode: {}", other),
        }
    }
}

#[derive(Default)]
struct InputBuffer {
    values: VecDeque<i32>,
}

impl InputBuffer {
    fn read(&mut self) -> i32 {
        self.values.pop_front().expect("no more inputs")
    }
}

#[derive(Default)]
struct OutputBuffer {
    values: VecDeque<i32>,
}

impl OutputBuffer {
    fn write(&mut self, value: i32) {
        self.values.push_back(value)
    }
}

struct Instruction {
    opcode: Opcode,
    param_modes: ParamModes,
}

enum Run {
    SetIp(usize),
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
    fn run(
        &self,
        memory: &mut [i32],
        ip: usize,
        input_buffer: &mut InputBuffer,
        output_buffer: &mut OutputBuffer,
    ) -> Run {
        match self.opcode {
            Opcode::Add => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = lhs + rhs;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Run::SetIp(ip + 4)
            }
            Opcode::Multiply => {
                let lhs_param = memory[ip + 1];
                let rhs_param = memory[ip + 2];
                let dst_param = memory[ip + 3];
                let lhs = self.param_modes.nth(0).read(lhs_param, memory);
                let rhs = self.param_modes.nth(1).read(rhs_param, memory);
                let value = lhs * rhs;
                self.param_modes.nth(2).write(dst_param, value, memory);
                Run::SetIp(ip + 4)
            }
            Opcode::Input => {
                let value = input_buffer.read();
                let param = memory[ip + 1];
                self.param_modes.nth(0).write(param, value, memory);
                Run::SetIp(ip + 2)
            }
            Opcode::Output => {
                let param = memory[ip + 1];
                let output = self.param_modes.nth(0).read(param, memory);
                output_buffer.write(output);
                Run::SetIp(ip + 2)
            }
            Opcode::Halt => Run::Halt,
        }
    }
}

fn run(memory: &mut [i32], input_buffer: &mut InputBuffer, output_buffer: &mut OutputBuffer) {
    let mut ip = 0;
    loop {
        let instruction = Instruction::decode(memory[ip]);
        match instruction.run(memory, ip, input_buffer, output_buffer) {
            Run::Halt => break,
            Run::SetIp(next_ip) => ip = next_ip,
        }
    }
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .unwrap();
    let mut memory = input_string
        .split(",")
        .map(|s| s.trim().parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    let mut input_buffer = InputBuffer::default();
    let mut output_buffer = OutputBuffer::default();
    input_buffer.values.push_back(1);
    run(&mut memory, &mut input_buffer, &mut output_buffer);
    println!("{}", output_buffer.values.pop_back().unwrap());
}
