mod mastermind;

use mastermind::mastermind_solver::candidate_handling::StrategyHandlerType;
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
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    println!("Hello, mastermind!");
    use self::mastermind::ConfigType;
    use self::mastermind::Game as mastermind;
    let configuration = ConfigType::new(4, 6);
    let mut game = mastermind::new(&configuration);

    // game.play();
    info!("checking strategy");
    let config = ConfigType::new_extended(5, 3);

    // 5 colors & 3 columns => ~2 min calculation time and 451 guesses // parallel: 20 seconds
    // 4 colors & 3 columns => ~2 seconds calculation time and 206 guesses

    // test_solver(&config, counts);
    let mut sht = StrategyHandlerType::new(&config);
    sht.solve();
    sht.verify();
}
