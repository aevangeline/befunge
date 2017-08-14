use std::char;
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Result;
use std::path::Path;
use std::vec::Vec;

use state::Direction::*;
use util;

extern crate rand;

// Location represents a specific place in the execution space
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Location {
    x: usize,
    y: usize,
}

impl Location {
    //step takes the location and moves in given a direction
    pub fn step(&self, direction: Direction) -> Location {
        let y = match direction {
            Up => self.y.wrapping_sub(1),
            Down => self.y.wrapping_add(1),
            _ => self.y,
        };

        let x = match direction {
            Right => self.x.wrapping_add(1),
            Left => self.x.wrapping_sub(1),
            _ => self.x,
        };

        Location { x: x, y: y }
    }

    //step_mut destructively moves this location in the given direction
    pub fn step_mut(&mut self, direction: Direction) -> Location {
        let next = self.step(direction);
        self.x = next.x;
        self.y = next.y;
        *self

    }

    // new creates a new Location object
    pub fn new(x: usize, y: usize) -> Location {
        Location { x: x, y: y }
    }
}

// Direction represents a specific direction for the interpreter to go
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Up => Down,
            Left => Right,
            Right => Left,
            Down => Up,
        }
    }

    pub fn up_or_down() -> Direction {
        if rand::random() { Up } else { Down }
    }

    pub fn right_or_left() -> Direction {
        if rand::random() { Right } else { Down }
    }
}


impl Default for Direction {
    fn default() -> Direction {
        Direction::Right
    }
}


impl rand::Rand for Direction {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        let choices = [Up, Down, Left, Right];
        rng.choose(&choices).unwrap().clone()

    }
}
// The current execution mode of this InterpreterState
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal, // This intepreter is in the middle of normal execution
    Exited, // This interpreter has reached a terminus for execution
    Quoted, // This interpreter is reading a quoted string
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Normal
    }
}

// A covenience wrapper around a Vec to produce a stack
#[derive(Default)]
struct Stack {
    stack: Vec<i64>,
}

impl Stack {
    // pop gets the top-most value from the stack, or 0
    pub fn pop(&mut self) -> i64 {
        self.stack.pop().unwrap_or(0)
    }

    // push adds a new number to the top of the stack
    pub fn push(&mut self, item: i64) {
        self.stack.push(item);
    }

    // duplicate_top takes the top most item on the stack (or 0) or pushes it on the stack
    pub fn duplicate_top(&mut self) {
        let value = self.pop();
        self.push(value);
        self.push(value);
    }

    // swap_top takes the top two elements from the stack and reverses them
    pub fn swap_top(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a);
        self.push(b);
    }
}


// State represents the current state of the map
#[derive(Default)]
pub struct State {
    initial_grid: Vec<Vec<char>>, // the state of the grid when loaded from the file
    grid_updates: HashMap<Location, char>, // any updates to the grid from runtime
    stack: Stack, // the current stack of values pushed from execution
    cursor: Location, // the instruction pointer that shows where we are in the grid
    direction: Direction, // the direction in which the cursor is moving
    execution_mode: Mode, // what execution mode is the interpreter currently in
}

impl State {
    //empty state produces a fresh, empty state object
    pub fn empty_state() -> State {
        Default::default()
    }

    //new_from_file produces a state containing the contents of the file
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<State> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut state = State::empty_state();
        for line in buf_reader.lines() {
            let l = line.unwrap_or_default();
            let mut vec = Vec::new();
            for ch in l.chars() {
                vec.push(ch);
            }
            state.initial_grid.push(vec)
        }
        Ok(state)
    }

    // value_at - returns the value at a given location
    pub fn value_at(&self, loc: Location) -> Option<char> {
        if self.grid_updates.contains_key(&loc) {
            return Some(self.grid_updates[&loc]);
        }

        if self.initial_grid.len() >= loc.y {
            return None;
        }

        if self.initial_grid[loc.y].len() >= loc.x {
            return None;
        }

        Some(self.initial_grid[loc.y][loc.x])
    }

    // current_value - gets the value at the current position
    pub fn current_value(&self) -> Option<char> {
        self.value_at(self.cursor.clone())
    }

    // set_value - sets the value at a given location
    pub fn set_value(&mut self, loc: Location, ch: char) {
        self.grid_updates.insert(loc, ch);
    }

    // increment_cursor moves our cursor to the next position - does not wrap
    fn increment_cursor(&mut self) -> Location {
        self.cursor.step_mut(self.direction)
    }

    // next_value increments the cursor and produces the next value if there is one
    fn next_value(&mut self) -> Option<char> {
        self.step_cursor();
        self.current_value()
    }

    // step_cursor moves the cursor to the next valid space
    pub fn step_cursor(&mut self) -> bool {
        if self.execution_mode == Mode::Exited {
            return false;
        }

        let start = self.cursor;
        while self.next_value().is_none() {
            if start == self.cursor {
                return false;
            }
        }

        true
    }

    // process_quoted reads the current character and pushes it onto the stack if needed
    pub fn process_quoted(&mut self, ch: char) {
        if ch == '"' {
            self.execution_mode = Mode::Normal;
            return;
        }
        self.stack.push(util::char_to_i64(ch));
    }


    // process_normal reads the current character and process it according to normal rules
    pub fn process_normal(&mut self, ch: char) {
        match ch {
            // push digits to the stack
            '0'...'9' | 'a'...'f' => push_digit(self, ch),

            // modal operators
            '"' => self.execution_mode = Mode::Quoted,
            '#' => {self.step_cursor();},

            // arithmetic operators
            '+' => addition(self),
            '-' => subtraction(self),
            '*' => multiply(self),
            '/' => divide(self),
            '%' => modulo(self),

            // logical operators
            '!' => logical_negation(self),
            '`' => greater_than(self),

            // directional operations
            '>' => self.direction = Right,
            '<' => self.direction = Left,
            '^' => self.direction = Up,
            'v' => self.direction = Down,
            '?' => self.direction = rand::random::<Direction>(),

            // branching operators
            '|' => veritical_if(self),
            '_' => horizontal_if(self),

            // stack manipulation operators
            '$' => {self.stack.pop();},
            ':' => self.stack.duplicate_top(),
            '\\' => self.stack.swap_top(),

            // input operators
            '&' => read_integer(self),
            '~' => read_char(self),

            // output operators
            '.' => print_digit(self),
            ',' => print_char(self),

            // load/store operations
            'p' => put(self),
            'g' => get(self),


            // end the program
            '@' => end_program(self),
            _ => (),
        };

    }
}

// read_char reads a character from the user
fn read_char(state : &mut State) {
    let mut input = String::new();
    let mut ch : char = 0 as char;
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Some(c) = input.chars().next(){
            ch = c;
        }
    }
    state.stack.push(util::char_to_i64(ch));
}

// read_integer reads a number from the commandline
fn read_integer(state : &mut State) {
    let mut input = String::new();
    if let Ok(_) = io::stdin().read_line(&mut input) {
        if let Ok(value) = input.trim().parse::<i64>() {
            state.stack.push(value);
            return;
        }
    }
    state.stack.push(0);
}

// put pops the values y, x, and v and stores value v at location {x,y}
fn put(state: &mut State) {
    let y = state.stack.pop();
    let x = state.stack.pop();
    let v = state.stack.pop();
    state.set_value(Location{x : x as usize, y : y as usize }, util::i64_to_char(v));
}

// get puts the value at {x, y} onto the stack
fn get(state: &mut State) {
    let y = state.stack.pop();
    let x = state.stack.pop();
    let v = util::char_to_i64(state.value_at(Location{x : x as usize, y : y as usize}).unwrap_or(0 as char));
}

// greater_than tests if b > a
fn greater_than(state : &mut State) {
    let a = state.stack.pop();
    let b = state.stack.pop();
    let mut result = 0;
    if b > a {
        result = 1;
    }
    state.stack.push(result);
}

// logical_negation tests if the top most value is 0
fn logical_negation(state : &mut State) {
    let mut negation = 0;
    if state.stack.pop() == 0 {
        negation = 1;
    }
    state.stack.push(negation);
}

// print_digit prints a number from the stack
fn print_digit(state: &mut State) {
    print!("{}", state.stack.pop());
}

// print_char prints the top value on the stack as a char
fn print_char(state: &mut State) {
    print!("{}", util::i64_to_char(state.stack.pop()));
}

// push_digit takes a character and pushs it onto the stack as a digit
fn push_digit(state: &mut State, ch: char) {
    if let Some(v) = ch.to_digit(16) {
        state.stack.push(v as i64);
    }
}

// addition performs addition on the two top values from the stack
fn addition(state: &mut State) {
    let sum = state.stack.pop() + state.stack.pop();
    state.stack.push(sum);
}

// subtraction performs b - a where a is popped from the stack before b
fn subtraction(state: &mut State) {
    let a = state.stack.pop();
    let b = state.stack.pop();
    state.stack.push(b - a);
}

// multiply performs a * b by popping two values from the stack
fn multiply(state: &mut State) {
    let product = state.stack.pop() * state.stack.pop();
    state.stack.push(product);
}

// divide performs b / a => does NOT user prompt
fn divide(state: &mut State) {
    let a = state.stack.pop();
    let b = state.stack.pop();
    state.stack.push(b / a);
}

// modulo performs b % a
fn modulo(state: &mut State) {
    let a = state.stack.pop();
    let b = state.stack.pop();
    state.stack.push(b % a);
}

// end_program sets the program to a terminated state
fn end_program(state: &mut State) {
    state.execution_mode = Mode::Exited;
}

// horizontal_if calculates a vertical branch
fn horizontal_if(state: &mut State) {
    state.direction = Left;
    if state.stack.pop() == 0 {
        state.direction = Right;
    }
}

// veritical_if calculates a horizontal branch
fn veritical_if(state: &mut State) {
    state.direction = Up;
    if state.stack.pop() == 0 {
        state.direction = Down;
    }
}
