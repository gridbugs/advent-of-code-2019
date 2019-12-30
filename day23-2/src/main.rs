use std::collections::VecDeque;
use std::io::Read;

#[derive(Debug)]
enum ParamMode {
    Positional,
    Immediate,
    Relative,
}

struct ParamArgs {
    param: i128,
    relative_base: i128,
}

impl ParamMode {
    fn from_i128(i128: i128) -> Self {
        match i128 {
            0 => Self::Positional,
            1 => Self::Immediate,
            2 => Self::Relative,
            other => panic!("unexpected param mode code: {}", other),
        }
    }
    fn read(
        &self,
        ParamArgs {
            param,
            relative_base,
        }: ParamArgs,
        memory: &[i128],
    ) -> i128 {
        match self {
            Self::Positional => {
                assert!(param >= 0);
                memory[param as usize]
            }
            Self::Immediate => param,
            Self::Relative => {
                let address = param + relative_base;
                assert!(address >= 0);
                memory[address as usize]
            }
        }
    }
    fn write(
        &self,
        ParamArgs {
            param,
            relative_base,
        }: ParamArgs,
        value: i128,
        memory: &mut [i128],
    ) {
        match self {
            Self::Positional => {
                assert!(param >= 0);
                memory[param as usize] = value;
            }
            Self::Immediate => panic!("attempted to write in immediate mode"),
            Self::Relative => {
                let address = param + relative_base;
                assert!(address >= 0);
                memory[address as usize] = value;
            }
        }
    }
}

#[derive(Debug)]
struct ParamModes {
    encoded: i128,
}

impl ParamModes {
    fn nth(&self, n: u32) -> ParamMode {
        ParamMode::from_i128((self.encoded / (10_i128.pow(n) as i128)) % 10)
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
    AdjustRelativeBase,
    Halt,
}

impl Opcode {
    fn from_i128(i128: i128) -> Self {
        match i128 {
            1 => Self::Add,
            2 => Self::Multiply,
            3 => Self::Input,
            4 => Self::Output,
            5 => Self::JumpIfTrue,
            6 => Self::JumpIfFalse,
            7 => Self::LessThan,
            8 => Self::Equals,
            9 => Self::AdjustRelativeBase,
            99 => Self::Halt,
            other => panic!("unexpected opcode: {}", other),
        }
    }
}

#[derive(Default, Debug)]
struct IoBuffer {
    values: VecDeque<i128>,
}

enum Echo {
    On,
    Off,
}

impl IoBuffer {
    fn read(&mut self) -> Option<i128> {
        self.values.pop_front()
    }
    fn drain<'a>(&'a mut self) -> impl 'a + Iterator<Item = i128> {
        self.values.drain(..)
    }
    fn drain_ascii_string(&mut self) -> String {
        let mut buffer = Vec::new();
        while let Some(&next) = self.values.front() {
            if next <= 127 {
                buffer.push(self.values.pop_front().unwrap() as u8);
            } else {
                break;
            }
        }
        String::from_utf8(buffer).unwrap()
    }
    fn write(&mut self, value: i128) {
        self.values.push_back(value)
    }
    fn write_ascii_string(&mut self, s: &str, echo: Echo) {
        match echo {
            Echo::On => print!("{}", s),
            Echo::Off => (),
        }
        for byte in s.bytes() {
            self.write(byte as i128);
        }
    }
    fn len(&self) -> usize {
        self.values.len()
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

enum Status {
    Running,
    WaitForInput,
    WroteOutput,
    Halt,
}

#[derive(Default)]
struct State {
    ip: usize,
    relative_base: i128,
}

impl Instruction {
    fn decode(encoded: i128) -> Self {
        let opcode = Opcode::from_i128(encoded % 100);
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
        memory: &mut [i128],
        state: &mut State,
        input_buffer: &mut IoBuffer,
        output_buffer: &mut IoBuffer,
    ) -> Status {
        let relative_base = state.relative_base;
        match self.opcode {
            Opcode::Add => {
                let lhs_param = memory[state.ip + 1];
                let rhs_param = memory[state.ip + 2];
                let dst_param = memory[state.ip + 3];
                let lhs = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: lhs_param,
                        relative_base,
                    },
                    memory,
                );
                let rhs = self.param_modes.nth(1).read(
                    ParamArgs {
                        param: rhs_param,
                        relative_base,
                    },
                    memory,
                );
                let value = lhs + rhs;
                self.param_modes.nth(2).write(
                    ParamArgs {
                        param: dst_param,
                        relative_base,
                    },
                    value,
                    memory,
                );
                state.ip += 4;
                Status::Running
            }
            Opcode::Multiply => {
                let lhs_param = memory[state.ip + 1];
                let rhs_param = memory[state.ip + 2];
                let dst_param = memory[state.ip + 3];
                let lhs = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: lhs_param,
                        relative_base,
                    },
                    memory,
                );
                let rhs = self.param_modes.nth(1).read(
                    ParamArgs {
                        param: rhs_param,
                        relative_base,
                    },
                    memory,
                );
                let value = lhs * rhs;
                self.param_modes.nth(2).write(
                    ParamArgs {
                        param: dst_param,
                        relative_base,
                    },
                    value,
                    memory,
                );
                state.ip += 4;
                Status::Running
            }
            Opcode::Input => {
                if let Some(value) = input_buffer.read() {
                    let param = memory[state.ip + 1];
                    self.param_modes.nth(0).write(
                        ParamArgs {
                            param,
                            relative_base,
                        },
                        value,
                        memory,
                    );
                    state.ip += 2;
                    Status::Running
                } else {
                    Status::WaitForInput
                }
            }
            Opcode::Output => {
                let param = memory[state.ip + 1];
                let output = self.param_modes.nth(0).read(
                    ParamArgs {
                        param,
                        relative_base,
                    },
                    memory,
                );
                output_buffer.write(output);
                state.ip += 2;
                Status::WroteOutput
            }
            Opcode::JumpIfTrue => {
                let cond_param = memory[state.ip + 1];
                let target_param = memory[state.ip + 2];
                let cond = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: cond_param,
                        relative_base,
                    },
                    memory,
                );
                if cond != 0 {
                    let target = self.param_modes.nth(1).read(
                        ParamArgs {
                            param: target_param,
                            relative_base,
                        },
                        memory,
                    );
                    state.ip = target as usize;
                    Status::Running
                } else {
                    state.ip += 3;
                    Status::Running
                }
            }
            Opcode::JumpIfFalse => {
                let cond_param = memory[state.ip + 1];
                let target_param = memory[state.ip + 2];
                let cond = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: cond_param,
                        relative_base,
                    },
                    memory,
                );
                if cond == 0 {
                    let target = self.param_modes.nth(1).read(
                        ParamArgs {
                            param: target_param,
                            relative_base,
                        },
                        memory,
                    );
                    state.ip = target as usize;
                    Status::Running
                } else {
                    state.ip += 3;
                    Status::Running
                }
            }
            Opcode::LessThan => {
                let lhs_param = memory[state.ip + 1];
                let rhs_param = memory[state.ip + 2];
                let dst_param = memory[state.ip + 3];
                let lhs = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: lhs_param,
                        relative_base,
                    },
                    memory,
                );
                let rhs = self.param_modes.nth(1).read(
                    ParamArgs {
                        param: rhs_param,
                        relative_base,
                    },
                    memory,
                );
                let value = (lhs < rhs) as i128;
                self.param_modes.nth(2).write(
                    ParamArgs {
                        param: dst_param,
                        relative_base,
                    },
                    value,
                    memory,
                );
                state.ip += 4;
                Status::Running
            }
            Opcode::Equals => {
                let lhs_param = memory[state.ip + 1];
                let rhs_param = memory[state.ip + 2];
                let dst_param = memory[state.ip + 3];
                let lhs = self.param_modes.nth(0).read(
                    ParamArgs {
                        param: lhs_param,
                        relative_base,
                    },
                    memory,
                );
                let rhs = self.param_modes.nth(1).read(
                    ParamArgs {
                        param: rhs_param,
                        relative_base,
                    },
                    memory,
                );
                let value = (lhs == rhs) as i128;
                self.param_modes.nth(2).write(
                    ParamArgs {
                        param: dst_param,
                        relative_base,
                    },
                    value,
                    memory,
                );
                state.ip += 4;
                Status::Running
            }
            Opcode::AdjustRelativeBase => {
                let param = memory[state.ip + 1];
                let adjust_by = self.param_modes.nth(0).read(
                    ParamArgs {
                        param,
                        relative_base,
                    },
                    memory,
                );
                state.relative_base += adjust_by;
                state.ip += 2;
                Status::Running
            }
            Opcode::Halt => Status::Halt,
        }
    }
}

struct IntcodeComputer {
    memory: Vec<i128>,
    state: State,
}

#[derive(Debug, PartialEq, Eq)]
enum StopStatus {
    WaitForInput,
    WroteOutput,
    Halt,
}

impl IntcodeComputer {
    fn new(program: &[i128]) -> Self {
        let mut memory = vec![0; 1 << 16];
        &mut memory[0..program.len()].copy_from_slice(program);
        Self {
            memory,
            state: State::default(),
        }
    }
    fn step(
        &mut self,
        input_buffer: &mut IoBuffer,
        output_buffer: &mut IoBuffer,
    ) -> Option<StopStatus> {
        let instruction = Instruction::decode(self.memory[self.state.ip]);
        match instruction.run(
            &mut self.memory,
            &mut self.state,
            input_buffer,
            output_buffer,
        ) {
            Status::Running => None,
            Status::Halt => Some(StopStatus::Halt),
            Status::WaitForInput => Some(StopStatus::WaitForInput),
            Status::WroteOutput => Some(StopStatus::WroteOutput),
        }
    }
    fn run(&mut self, input_buffer: &mut IoBuffer, output_buffer: &mut IoBuffer) -> StopStatus {
        loop {
            if let Some(stop_status) = self.step(input_buffer, output_buffer) {
                return stop_status;
            }
        }
    }
}

enum ComputerStep {
    Halt,
    SendPacket(Packet),
    ReceivedEmptyQueue,
}

struct Packet {
    destination: u8,
    x: i128,
    y: i128,
}

struct Computer {
    input_buffer: IoBuffer,
    output_buffer: IoBuffer,
    computer: IntcodeComputer,
}

impl Computer {
    fn new(program: &[i128], address: u8) -> Self {
        let mut s = Self {
            input_buffer: IoBuffer::default(),
            output_buffer: IoBuffer::default(),
            computer: IntcodeComputer::new(program),
        };
        s.input_buffer.write(address as i128);
        s
    }
    fn step(&mut self) -> Option<ComputerStep> {
        match self
            .computer
            .step(&mut self.input_buffer, &mut self.output_buffer)
        {
            None => return None,
            Some(StopStatus::Halt) => return Some(ComputerStep::Halt),
            Some(StopStatus::WaitForInput) => {
                self.input_buffer.write(-1);
                let stop_status = self
                    .computer
                    .step(&mut self.input_buffer, &mut self.output_buffer);
                assert_eq!(stop_status, None);
                return Some(ComputerStep::ReceivedEmptyQueue);
            }
            Some(StopStatus::WroteOutput) => {
                if self.output_buffer.len() == 3 {
                    let destination = self.output_buffer.read().unwrap();
                    assert!(destination >= 0 && destination < 256);
                    let destination = destination as u8;
                    let x = self.output_buffer.read().unwrap();
                    let y = self.output_buffer.read().unwrap();
                    return Some(ComputerStep::SendPacket(Packet { destination, x, y }));
                } else {
                    return None;
                }
            }
        }
    }
    fn receive(&mut self, x: i128, y: i128) {
        self.input_buffer.write(x);
        self.input_buffer.write(y);
    }
    fn is_queue_empty(&self) -> bool {
        self.input_buffer.is_empty()
    }
}

#[derive(Default)]
struct Nat {
    x: i128,
    y: i128,
}

struct Network {
    computers: Vec<Computer>,
    nat: Nat,
    last_nat_y: Option<i128>,
    idle_count: u64,
}

impl Network {
    fn new(program: &[i128]) -> Self {
        Self {
            computers: (0..50)
                .map(|address| Computer::new(program, address))
                .collect(),
            nat: Nat::default(),
            last_nat_y: None,
            idle_count: 0,
        }
    }
    fn step(&mut self) -> Option<i128> {
        let mut empty_count = 0;
        for i in 0..self.computers.len() {
            match self.computers[i].step() {
                None => (),
                Some(ComputerStep::Halt) => println!("{} has halted", i),
                Some(ComputerStep::ReceivedEmptyQueue) => (), //empty_count += 1,
                Some(ComputerStep::SendPacket(packet)) => {
                    if packet.destination == 255 {
                        self.nat.x = packet.x;
                        self.nat.y = packet.y;
                        println!("write to nat y {}", packet.y);
                    }
                    if let Some(computer) = self.computers.get_mut(packet.destination as usize) {
                        computer.receive(packet.x, packet.y);
                    }
                }
            }
            if self.computers[i].is_queue_empty() {
                empty_count += 1;
            }
        }
        if empty_count == self.computers.len() {
            self.idle_count += 1;
        } else {
            self.idle_count = 0;
        }
        let mut ret = None;
        if self.idle_count == 10000 {
            println!("network is idle, sending {}", self.nat.y);
            self.computers[0].receive(self.nat.x, self.nat.y);
            if self.last_nat_y == Some(self.nat.y) {
                ret = self.last_nat_y;
            }
            self.last_nat_y = Some(self.nat.y);
        }
        ret
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
        .map(|s| s.trim().parse::<i128>().unwrap())
        .collect::<Vec<_>>();
    let mut network = Network::new(&program);
    loop {
        if let Some(y) = network.step() {
            println!("{}", y);
            break;
        }
    }
}
