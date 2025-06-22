mod mastermind;
#[macro_use]
extern crate fstrings;

// use mastermind::ConfigType;
use std::fmt;

fn main() {
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
