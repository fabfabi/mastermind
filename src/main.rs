mod mastermind;

#[macro_use]
extern crate fstrings;
#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

use chrono::{DateTime, Utc};

use self::mastermind::ConfigType;
use clap::Parser;
use std::fs::File;

// use mastermind::ConfigType;
// use std::fmt;

/// simple parser, see https://rust-cli.github.io/book/tutorial/cli-args.html
#[derive(Parser)]
struct Cli {
    job: String,
    n_columns: Option<usize>,
    n_colors: Option<u8>,
}
impl Cli {
    fn run(&self) {
        match self.job.as_str() {
            "play" => Cli::play(),
            "solve" => Cli::solve(self.n_columns, self.n_colors),
            _ => error!("Unknown input {}. Please use 'play' or 'solve'", self.job),
        }
    }
    fn play() {
        use self::mastermind::Game as mastermind;
        let configuration = ConfigType::new(4, 6);
        let mut game = mastermind::new(&configuration);

        game.play();
    }

    fn solve(n_columns: Option<usize>, n_colors: Option<u8>) {
        use mastermind::mastermind_solver::StrategyHandlerType;
        let config = ConfigType::new_extended(n_colors.unwrap_or(5), n_columns.unwrap_or(3));

        // 5 colors & 3 columns => ~2 min calculation time and 451 guesses // parallel: 20 seconds
        // 4 colors & 3 columns => ~2 seconds calculation time and 206 guesses

        // test_solver(&config, counts);
        let mut sht = StrategyHandlerType::new(&config);
        sht.solve();
        sht.verify();
        sht.show();
    }
}

fn main() {
    let now: DateTime<Utc> = Utc::now();
    let formatted_date = now.format("%Y%m%d").to_string();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(formatted_date + "_mastermind.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("Hello, Mastermind!");
    let args = Cli::parse();
    args.run();
}
