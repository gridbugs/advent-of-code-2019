use std::collections::HashMap;
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

struct Computer {
    input_buffer: IoBuffer,
    output_buffer: IoBuffer,
    computer: IntcodeComputer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn parse(s: &str) -> Self {
        match s {
            "north" => Self::North,
            "east" => Self::East,
            "south" => Self::South,
            "west" => Self::West,
            other => panic!("not direction: {}", other),
        }
    }
    fn to_str(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }
    fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

#[derive(Debug)]
struct Room {
    exits: Vec<Direction>,
    name: String,
    description: String,
    items: Vec<String>,
}

impl Room {
    fn parse(s: &str) -> Self {
        //println!("parsing #####\n{}\n#####", s);
        let trimmed = s.trim();
        let lines = trimmed.lines().collect::<Vec<_>>();
        let name = lines[0].split(" ").nth(1).unwrap();
        let description = lines[1];
        assert_eq!(lines[3], "Doors here lead:");
        let mut exits = Vec::new();
        for &line in &lines[4..] {
            if line.starts_with("- ") {
                exits.push(Direction::parse(
                    &line.split(' ').skip(1).collect::<Vec<_>>().join(" "),
                ))
            } else {
                break;
            }
        }
        let mut items = Vec::new();
        let item_head_offset = 4 + exits.len() + 1;
        if lines[item_head_offset] == "Items here:" {
            for &line in &lines[item_head_offset + 1..] {
                if line.starts_with("- ") {
                    items.push(line.split(' ').skip(1).collect::<Vec<_>>().join(" "));
                } else {
                    break;
                }
            }
        }
        Self {
            exits,
            name: name.to_string(),
            description: description.to_string(),
            items,
        }
    }
}

type Path = Vec<(String, Direction)>;

#[derive(Debug)]
struct Ship {
    path_from_hull_to_security: Path,
    paths_to_each_item: HashMap<String, Path>,
    pressure_room_direction: Direction,
}

impl Computer {
    fn new(program: &[i128]) -> Self {
        Self {
            input_buffer: IoBuffer::default(),
            output_buffer: IoBuffer::default(),
            computer: IntcodeComputer::new(program),
        }
    }
    fn explore(&mut self) -> Ship {
        let prompt = self.run_until_prompt();
        let room = Room::parse(&prompt);
        let mut path_from_start_to_current: Vec<(String, Direction)> = Vec::new();
        let mut exits_to_explore = Vec::new();
        exits_to_explore.extend(room.exits.iter().map(|&exit| (room.name.clone(), exit)));
        let mut item_name_to_path = HashMap::new();
        let mut maybe_path_to_security = None;
        let mut maybe_pressure_room_direction = None;
        while let Some((room_name_to_explore_from, exit_to_explore)) = exits_to_explore.pop() {
            while let Some((room_name, direction_to_get_here)) = path_from_start_to_current.last() {
                if room_name == &room_name_to_explore_from {
                    break;
                }
                self.input_buffer.write_ascii_string(
                    &format!("{}\n", direction_to_get_here.opposite().to_str()),
                    Echo::Off,
                );
                let prompt = self.run_until_prompt();
                let _room = Room::parse(&prompt);
                path_from_start_to_current.pop();
            }
            self.input_buffer
                .write_ascii_string(&format!("{}\n", exit_to_explore.to_str()), Echo::Off);
            let prompt = self.run_until_prompt();
            let room = Room::parse(&prompt);
            path_from_start_to_current.push((room.name.clone(), exit_to_explore));
            if room.name == "Security" {
                let mut path_to_security = path_from_start_to_current.clone();
                assert_eq!(room.exits.len(), 2);
                let &exit = room
                    .exits
                    .iter()
                    .filter(|&&exit| exit != exit_to_explore.opposite())
                    .next()
                    .unwrap();
                maybe_path_to_security = Some(path_to_security);
                maybe_pressure_room_direction = Some(exit);
            }
            exits_to_explore.extend(room.exits.iter().filter_map(|&exit| {
                if exit == exit_to_explore.opposite() {
                    None
                } else if room.name == "Security" && exit == Direction::North {
                    None
                } else {
                    Some((room.name.clone(), exit))
                }
            }));
            for item in room.items.iter() {
                item_name_to_path.insert(item.clone(), path_from_start_to_current.clone());
            }
        }
        while let Some((_, direction_to_get_here)) = path_from_start_to_current.pop() {
            self.input_buffer.write_ascii_string(
                &format!("{}\n", direction_to_get_here.opposite().to_str()),
                Echo::Off,
            );
            let prompt = self.run_until_prompt();
            let room = Room::parse(&prompt);
        }
        Ship {
            path_from_hull_to_security: maybe_path_to_security.unwrap(),
            paths_to_each_item: item_name_to_path,
            pressure_room_direction: maybe_pressure_room_direction.unwrap(),
        }
    }
    fn run_until_prompt(&mut self) -> String {
        loop {
            match self
                .computer
                .run(&mut self.input_buffer, &mut self.output_buffer)
            {
                StopStatus::Halt => panic!(
                    "unexpected halt: {}",
                    self.output_buffer.drain_ascii_string()
                ),
                StopStatus::WroteOutput => (),
                StopStatus::WaitForInput => return self.output_buffer.drain_ascii_string(),
            }
        }
    }
    fn follow_path(&mut self, path: &Path) {
        for (room_name, direction) in path {
            self.input_buffer
                .write_ascii_string(&format!("{}\n", direction.to_str()), Echo::On);
            let prompt = self.run_until_prompt();
            let room = Room::parse(&prompt);
            assert_eq!(room_name, &room.name);
            println!("{}", prompt);
        }
    }
    fn take(&mut self, item: &str) {
        self.input_buffer
            .write_ascii_string(&format!("take {}\n", item), Echo::On);
        let prompt = self.run_until_prompt();
        println!("{}", prompt);
    }
    fn drop(&mut self, item: &str) {
        self.input_buffer
            .write_ascii_string(&format!("drop {}\n", item), Echo::On);
        let prompt = self.run_until_prompt();
        println!("{}", prompt);
    }
    fn follow_path_rev(&mut self, path: &Path) {
        for (_room_name, direction) in path.iter().rev() {
            self.input_buffer
                .write_ascii_string(&format!("{}\n", direction.opposite().to_str()), Echo::On);
            let prompt = self.run_until_prompt();
            let _room = Room::parse(&prompt);
            println!("{}", prompt);
        }
    }
    fn collect_item(&mut self, ship: &Ship, item: &str) {
        let path = ship.paths_to_each_item.get(item).unwrap();
        self.follow_path(path);
        self.take(item);
        self.follow_path_rev(path);
    }
    fn go_to_security_checkpoint(&mut self, ship: &Ship) {
        self.follow_path(&ship.path_from_hull_to_security);
    }
    fn inv(&mut self) -> String {
        self.input_buffer.write_ascii_string("inv\n", Echo::On);
        self.run_until_prompt()
    }
    fn collect_all_items(&mut self, ship: &Ship) {
        for item in ALL_ITEMS {
            self.collect_item(ship, item);
        }
    }
    fn drop_all_items_in_security_room(&mut self, ship: &Ship) {
        self.follow_path(&ship.path_from_hull_to_security);
        for item in ALL_ITEMS {
            self.drop(item);
        }
    }
    fn try_to_pass_pressure_room(&mut self, ship: &Ship) -> PressureRoom {
        self.input_buffer.write_ascii_string(
            &format!("{}\n", ship.pressure_room_direction.to_str()),
            Echo::On,
        );
        let prompt = self.run_until_prompt();
        const HEAVY_MESSAGE: &str =
            "Alert! Droids on this ship are heavier than the detected value!";
        const LIGHT_MESSAGE: &str =
            "Alert! Droids on this ship are lighter than the detected value!";
        if prompt.contains(HEAVY_MESSAGE) {
            PressureRoom::Heavy
        } else if prompt.contains(LIGHT_MESSAGE) {
            PressureRoom::Light
        } else {
            PressureRoom::Correct
        }
    }
    fn take_item_subset(&mut self, subset: u8) {
        for i in 0..ALL_ITEMS.len() {
            if subset & (1 << i) != 0 {
                self.take(ALL_ITEMS[i]);
            }
        }
    }
    fn drop_item_subset(&mut self, subset: u8) {
        for i in 0..ALL_ITEMS.len() {
            if subset & (1 << i) != 0 {
                self.drop(ALL_ITEMS[i]);
            }
        }
    }
    fn try_to_pass_pressure_room_with_subset(&mut self, ship: &Ship, subset: u8) -> PressureRoom {
        self.take_item_subset(subset);
        let result = self.try_to_pass_pressure_room(ship);
        if let PressureRoom::Correct = result {
            panic!("it worked {}", subset);
        }
        self.drop_item_subset(subset);
        result
    }
    fn try_to_pass_pressure_room_with_all_subsets(&mut self, ship: &Ship) {
        for i in 0..=255 {
            self.try_to_pass_pressure_room_with_subset(ship, i);
        }
    }
}

enum PressureRoom {
    Heavy,
    Light,
    Correct,
}

const ALL_ITEMS: &[&str] = &[
    "astrolabe",
    "bowl of rice",
    "cake",
    "fuel cell",
    "monolith",
    "ornament",
    "planetoid",
    "shell",
];

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
    let mut computer = Computer::new(&program);
    let ship = computer.explore();
    computer.collect_all_items(&ship);
    computer.drop_all_items_in_security_room(&ship);
    computer.try_to_pass_pressure_room_with_all_subsets(&ship);
}
