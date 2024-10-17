#![allow(dead_code, unused_variables)]
//#![warn(clippy::pedantic)]
mod cli;
mod heatmap;
mod state;
mod types;

const DEFAULT_HEIGHT: usize = 7;
const DEFAULT_WIDTH: usize = 9;
const DEFAULT_SHIPS: [usize; 5] = [2, 3, 3, 4, 5];

fn main() {
    let mut state = state::State::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, &DEFAULT_SHIPS);

    cli::main_loop(&mut state);
}
