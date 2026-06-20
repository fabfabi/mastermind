mod mastermind;
#[macro_use]
extern crate fstrings;
#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

// use mastermind::ConfigType;
use std::fmt;

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    println!("Hello, world!");
    use self::mastermind::ConfigType;
    use self::mastermind::Game as mastermind;
    let configuration = ConfigType {
        columns: 4,
        colors: 6,
    };
    let mut game = mastermind::new(&configuration);

    game.play();
}
