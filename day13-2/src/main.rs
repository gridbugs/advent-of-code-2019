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

impl IoBuffer {
    fn read(&mut self) -> Option<i128> {
        self.values.pop_front()
    }
    fn write(&mut self, value: i128) {
        self.values.push_back(value)
    }
    fn len(&self) -> usize {
        self.values.len()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

impl Tile {
    fn from_i128(i128: i128) -> Self {
        match i128 {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::HorizontalPaddle,
            4 => Self::Ball,
            _ => panic!("unexpected tile"),
        }
    }
    fn to_char(self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Wall => '#',
            Self::Block => '%',
            Self::HorizontalPaddle => '=',
            Self::Ball => 'o',
        }
    }
}

struct Grid {
    cells: Vec<Tile>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        let cells = vec![Tile::Empty; width * height];
        Self {
            cells,
            width,
            height,
        }
    }
    fn get_mut(&mut self, Coord { x, y }: Coord) -> &mut Tile {
        assert!(x >= 0);
        assert!(y >= 0);
        assert!((x as usize) < self.width);
        assert!((y as usize) < self.height);
        let index = self.width * y as usize + x as usize;
        &mut self.cells[index]
    }
    fn count_blocks(&self) -> usize {
        self.cells.iter().filter(|&&c| c == Tile::Block).count()
    }
    fn tile_coord(&self, query_tile: Tile) -> Coord {
        for (i, row) in self.cells.chunks_exact(self.width).enumerate() {
            for (j, &tile) in row.iter().enumerate() {
                if tile == query_tile {
                    return Coord {
                        x: j as i32,
                        y: i as i32,
                    };
                }
            }
        }
        panic!("no ball");
    }
    fn render(&self) {
        for row in self.cells.chunks_exact(self.width) {
            for tile in row {
                print!("{}", tile.to_char());
            }
            println!("");
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

const WIDTH: usize = 100;
const HEIGHT: usize = 40;

fn run(program: &[i128]) -> i128 {
    let mut computer = IntcodeComputer::new(&program);
    computer.memory[0] = 2;
    let mut input_buffer = IoBuffer::default();
    let mut output_buffer = IoBuffer::default();
    let mut grid = Grid::new(WIDTH, HEIGHT);
    let mut score = 0;
    loop {
        match computer.run(&mut input_buffer, &mut output_buffer) {
            StopStatus::Halt => return score,
            StopStatus::WaitForInput => {
                grid.render();
                let ball_coord = grid.tile_coord(Tile::Ball);
                let paddle_coord = grid.tile_coord(Tile::HorizontalPaddle);
                use std::cmp::Ordering;
                let input = match ball_coord.x.cmp(&paddle_coord.x) {
                    Ordering::Equal => 0,
                    Ordering::Less => -1,
                    Ordering::Greater => 1,
                };
                input_buffer.write(input);
            }
            StopStatus::WroteOutput => {
                if output_buffer.len() == 3 {
                    let x = output_buffer.read().unwrap() as i32;
                    let y = output_buffer.read().unwrap() as i32;
                    let coord = Coord { x, y };
                    if coord == (Coord { x: -1, y: 0 }) {
                        score = output_buffer.read().unwrap();
                        println!("score: {}", score);
                    } else {
                        let tile = Tile::from_i128(output_buffer.read().unwrap());
                        *grid.get_mut(coord) = tile;
                    }
                }
            }
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
        .map(|s| s.trim().parse::<i128>().unwrap())
        .collect::<Vec<_>>();
    let block_count = run(&program);
    println!("{}", block_count);
}
