mod state;
mod matcher;
mod util;
extern crate regex;
extern crate rand;
extern crate clap;
use clap::{App, Arg};
fn main() {
    let args = App::new("Befunge")
        .version("0.1.0")
        .author("Oreoluwa Babarinsa <oreoluwa@babarinsa.me>")
        .about("Interprets Befunge-9x programs")
        .arg(Arg::with_name("SOURCE")
             .help("the befunge source file")
             .required(true)
             .index(1))
        .get_matches();

    let fname = args.value_of("SOURCE").unwrap();
    match state::State::new_from_file(fname).as_mut() {
        Ok(state) => state.run(),
        Err(e) => eprintln!("Befunge error : {}", e),
    }

}
