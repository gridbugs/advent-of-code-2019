use std::collections::{HashSet, VecDeque};
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

impl IoBuffer {
    fn read(&mut self) -> Option<i128> {
        self.values.pop_front()
    }
    fn write(&mut self, value: i128) {
        self.values.push_back(value)
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
    fn run(&mut self, input_buffer: &mut IoBuffer, output_buffer: &mut IoBuffer) -> StopStatus {
        loop {
            let instruction = Instruction::decode(self.memory[self.state.ip]);
            match instruction.run(
                &mut self.memory,
                &mut self.state,
                input_buffer,
                output_buffer,
            ) {
                Status::Running => (),
                Status::Halt => return StopStatus::Halt,
                Status::WaitForInput => return StopStatus::WaitForInput,
                Status::WroteOutput => return StopStatus::WroteOutput,
            }
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct Coord {
    x: i32,
    y: i32,
}

impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Direction {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

impl Direction {
    fn to_i128(self) -> i128 {
        self as i128
    }
    fn to_unit_coord(self) -> Coord {
        match self {
            Self::North => Coord { x: 0, y: -1 },
            Self::South => Coord { x: 0, y: 1 },
            Self::West => Coord { x: -1, y: 0 },
            Self::East => Coord { x: 1, y: 0 },
        }
    }
    fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum DroidStatus {
    HitWall = 0,
    Moved = 1,
    MovedToGoal = 2,
}

impl DroidStatus {
    fn from_i128(i128: i128) -> Self {
        assert!(i128 >= 0 && i128 <= 2);
        unsafe { std::mem::transmute(i128 as u8) }
    }
}

struct Node {
    destination: Coord,
    path_from_start: Vec<Direction>,
}

struct Droid {
    computer: IntcodeComputer,
    input_buffer: IoBuffer,
    output_buffer: IoBuffer,
}

impl Droid {
    fn new(program: &[i128]) -> Self {
        Self {
            computer: IntcodeComputer::new(program),
            input_buffer: IoBuffer::default(),
            output_buffer: IoBuffer::default(),
        }
    }
    fn run(&mut self) -> StopStatus {
        self.computer
            .run(&mut self.input_buffer, &mut self.output_buffer)
    }
    fn step(&mut self, direction: Direction) -> DroidStatus {
        self.input_buffer.write(direction.to_i128());
        let stop_status = self.run();
        assert_eq!(stop_status, StopStatus::WroteOutput);
        DroidStatus::from_i128(self.output_buffer.read().unwrap())
    }
    fn follow_directions<I: IntoIterator<Item = Direction>>(&mut self, directions: I) {
        for direction in directions {
            let droid_status = self.step(direction);
            assert_eq!(droid_status, DroidStatus::Moved);
        }
    }
}

fn run(program: &[i128]) -> usize {
    let mut droid = Droid::new(program);
    let mut seen_set = HashSet::new();
    let mut queue = VecDeque::new();
    seen_set.insert(Coord { x: 0, y: 0 });
    queue.push_back(Node {
        destination: Coord { x: 0, y: 0 },
        path_from_start: Vec::new(),
    });
    while let Some(Node {
        destination,
        path_from_start,
    }) = queue.pop_front()
    {
        droid.follow_directions(path_from_start.iter().cloned());
        for &direction in &ALL_DIRECTIONS {
            let next_destination = destination + direction.to_unit_coord();
            if !seen_set.contains(&next_destination) {
                seen_set.insert(next_destination);
                let status = droid.step(direction);
                match status {
                    DroidStatus::HitWall => (),
                    DroidStatus::MovedToGoal => return path_from_start.len() + 1,
                    DroidStatus::Moved => {
                        droid.step(direction.opposite());
                        queue.push_back(Node {
                            destination: next_destination,
                            path_from_start: {
                                let mut path = path_from_start.clone();
                                path.push(direction);
                                path
                            },
                        });
                    }
                }
            }
        }
        droid.follow_directions(
            path_from_start
                .iter()
                .rev()
                .cloned()
                .map(Direction::opposite),
        );
    }
    panic!("no path to goal");
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
    let distance = run(&program);
    println!("{}", distance);
}
