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
    fn drain<'a>(&'a mut self) -> impl 'a + Iterator<Item = i128> {
        self.values.drain(..)
    }
    fn drain_ascii_string(&mut self) -> String {
        let bytes = self.drain().map(|x| x as u8).collect::<Vec<_>>();
        String::from_utf8(bytes).unwrap()
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

const UNIT_COORDS: [Coord; 4] = [
    Coord { x: 0, y: 1 },
    Coord { x: 0, y: -1 },
    Coord { x: 1, y: 0 },
    Coord { x: -1, y: 0 },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Scaffold,
}

struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    fn get(&self, Coord { x, y }: Coord) -> Option<Cell> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return None;
        }
        let index = y as usize * self.width + x as usize;
        Some(self.cells[index])
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
    fn turn(self, turn: Turn) -> Self {
        match (self, turn) {
            (Self::North, Turn::Left) => Self::West,
            (Self::North, Turn::Right) => Self::East,
            (Self::East, Turn::Left) => Self::North,
            (Self::East, Turn::Right) => Self::South,
            (Self::South, Turn::Left) => Self::East,
            (Self::South, Turn::Right) => Self::West,
            (Self::West, Turn::Left) => Self::South,
            (Self::West, Turn::Right) => Self::North,
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

#[derive(Debug, Clone, Copy)]
struct Robot {
    location: Coord,
    facing: Direction,
}

fn parse_grid(s: &str) -> (Grid, Robot) {
    let mut width = 0;
    let mut height = 0;
    let mut cells = Vec::with_capacity(s.len());
    let mut robot = None;
    for (y, line) in s.lines().enumerate() {
        if line.len() == 0 {
            break;
        }
        width = line.len();
        for (x, c) in line.chars().enumerate() {
            cells.push(match c {
                '#' => Cell::Scaffold,
                '^' | '<' | '>' | 'v' => {
                    let facing = match c {
                        '^' => Direction::North,
                        '>' => Direction::East,
                        'v' => Direction::South,
                        '<' => Direction::West,
                        _ => unreachable!(),
                    };
                    robot = Some(Robot {
                        location: Coord {
                            x: x as i32,
                            y: y as i32,
                        },
                        facing,
                    });
                    Cell::Scaffold
                }
                '.' => Cell::Empty,
                _ => panic!(),
            });
        }
        height += 1;
    }
    (
        Grid {
            width,
            height,
            cells,
        },
        robot.unwrap(),
    )
}

#[derive(Clone, Copy, Debug)]
enum Turn {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Turn(Turn),
    Forward,
}

#[derive(Clone, Copy, Debug)]
enum CompressedStep {
    Turn(Turn),
    Forward(usize),
}

fn compress_steps(steps: &[Step]) -> Vec<CompressedStep> {
    let mut compressed_steps = Vec::new();
    for &step in steps {
        let compressed_step = match step {
            Step::Turn(turn) => compressed_steps.push(CompressedStep::Turn(turn)),
            Step::Forward => {
                if let Some(CompressedStep::Forward(ref mut distance)) = compressed_steps.last_mut()
                {
                    *distance += 1;
                } else {
                    compressed_steps.push(CompressedStep::Forward(1));
                }
            }
        };
    }
    compressed_steps
}

fn full_step_sequence(grid: &Grid, mut robot: Robot) -> Vec<Step> {
    let mut steps = Vec::new();
    loop {
        let next_coord = robot.location + robot.facing.to_unit_coord();
        if let Some(Cell::Scaffold) = grid.get(next_coord) {
            steps.push(Step::Forward);
            robot.location = next_coord;
            continue;
        }
        let left_coord = robot.location + robot.facing.turn(Turn::Left).to_unit_coord();
        if let Some(Cell::Scaffold) = grid.get(left_coord) {
            steps.push(Step::Turn(Turn::Left));
            robot.facing = robot.facing.turn(Turn::Left);
            continue;
        }
        let right_coord = robot.location + robot.facing.turn(Turn::Right).to_unit_coord();
        if let Some(Cell::Scaffold) = grid.get(right_coord) {
            steps.push(Step::Turn(Turn::Right));
            robot.facing = robot.facing.turn(Turn::Right);
            continue;
        }
        return steps;
    }
}

fn build_string_map(program: &[i128]) -> String {
    let mut input_buffer = IoBuffer::default();
    let mut output_buffer = IoBuffer::default();
    let mut ascii_buffer = Vec::new();
    let mut computer = IntcodeComputer::new(program);
    loop {
        match computer.run(&mut input_buffer, &mut output_buffer) {
            StopStatus::Halt => break,
            StopStatus::WaitForInput => panic!("unexpected wait for input"),
            StopStatus::WroteOutput => {
                let ascii = output_buffer.read().unwrap();
                ascii_buffer.push(ascii as u8);
            }
        }
    }
    String::from_utf8(ascii_buffer).unwrap()
}

struct InstructRobot {
    input_buffer: IoBuffer,
    output_buffer: IoBuffer,
    computer: IntcodeComputer,
}

impl InstructRobot {
    fn new(program: &[i128]) -> Self {
        Self {
            input_buffer: IoBuffer::default(),
            output_buffer: IoBuffer::default(),
            computer: IntcodeComputer::new(program),
        }
    }
    fn input_str(&mut self, input: &str) {
        println!("# input: {}", input);
        for byte in input.bytes() {
            self.input_buffer.write(byte as i128);
        }
    }
    fn run(&mut self) -> StopStatus {
        self.computer
            .run(&mut self.input_buffer, &mut self.output_buffer)
    }
    fn drain_ascii_string(&mut self) -> String {
        self.output_buffer.drain_ascii_string()
    }
    fn run_until_wait_for_input(&mut self) {
        loop {
            match self.run() {
                StopStatus::Halt => panic!("unexpected halt"),
                StopStatus::WaitForInput => break,
                StopStatus::WroteOutput => (),
            }
        }
    }
}

const MAIN: &str = "A,B,A,C,B,C,B,C,A,C\n";
const PROG_A: &str = "L,10,R,12,R,12\n";
const PROG_B: &str = "R,6,R,10,L,10\n";
const PROG_C: &str = "R,10,L,10,L,12,R,6\n";

fn instruct_robot(program: &[i128]) {
    let mut robot = InstructRobot::new(program);
    robot.run_until_wait_for_input();
    println!("{}", robot.drain_ascii_string());
    robot.input_str(MAIN);
    robot.run_until_wait_for_input();
    println!("{}", robot.drain_ascii_string());
    robot.input_str(PROG_A);
    robot.run_until_wait_for_input();
    println!("{}", robot.drain_ascii_string());
    robot.input_str(PROG_B);
    robot.run_until_wait_for_input();
    println!("{}", robot.drain_ascii_string());
    robot.input_str(PROG_C);
    robot.run_until_wait_for_input();
    println!("{}", robot.drain_ascii_string());
    robot.input_str("n\n");
    loop {
        match robot.run() {
            StopStatus::Halt => break,
            StopStatus::WaitForInput => panic!("unexpected wait for input"),
            StopStatus::WroteOutput => {
                if robot.output_buffer.len() == 1 {
                    println!("{}", robot.output_buffer.read().unwrap());
                } else {
                    print!("{}", robot.drain_ascii_string());
                }
            }
        }
    }
    println!("");
}

fn main() {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .unwrap();
    let mut program = input_string
        .split(",")
        .map(|s| s.trim().parse::<i128>().unwrap())
        .collect::<Vec<_>>();
    let map_string = build_string_map(&program);
    let (map_grid, robot) = parse_grid(&map_string);
    let mut intersections = HashSet::new();
    for i in 0..map_grid.height {
        'inner: for j in 0..map_grid.width {
            let coord = Coord {
                x: j as i32,
                y: i as i32,
            };
            if map_grid.get(coord).unwrap() == Cell::Scaffold {
                for &unit_coord in &UNIT_COORDS {
                    match map_grid.get(coord + unit_coord) {
                        Some(Cell::Scaffold) => (),
                        _ => continue 'inner,
                    }
                }
                intersections.insert(coord);
            }
        }
    }
    let full_steps = full_step_sequence(&map_grid, robot);
    let compressed_steps = compress_steps(&full_steps);
    // modify the program to instruct the robot
    program[0] = 2;
    instruct_robot(&program);
    for &compressed_step in &compressed_steps {
        match compressed_step {
            CompressedStep::Forward(distance) => print!("{}, ", distance),
            CompressedStep::Turn(Turn::Left) => print!("L, "),
            CompressedStep::Turn(Turn::Right) => print!("R, "),
        }
    }
}

// L, 10, R, 12, R, 12, R, 6, R, 10, L, 10, L, 10, R, 12, R, 12, R, 10, L, 10, L, 12, R, 6, R, 6,
// R, 10, L, 10, R, 10, L, 10, L, 12, R, 6, R, 6, R, 10, L, 10, R, 10, L, 10, L, 12, R, 6, L, 10,
// R, 12, R, 12, R, 10, L, 10, L, 12, R, 6,
//
// ########
//
// A: L, 10, R, 12, R, 12,
// B: R, 6, R, 10, L, 10,
// A: L, 10, R, 12, R, 12,
// C: R, 10, L, 10, L, 12, R, 6,
// B: R, 6, R, 10, L, 10,
// C: R, 10, L, 10, L, 12, R, 6,
// B: R, 6, R, 10, L, 10,
// C: R, 10, L, 10, L, 12, R, 6,
// A: L, 10, R, 12, R, 12,
// C: R, 10, L, 10, L, 12, R, 6,
//
// ABACBCBCAC
//
// ########
//
//  A: L, 10, R, 12, R, 12,
//
//  B: R, 6, R, 10, L, 10,
//
//  C: R, 10, L, 10, L, 12, R, 6,
//
