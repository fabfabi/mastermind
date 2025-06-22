mod mastermind;
#[macro_use]
extern crate fstrings;

// use mastermind::ConfigType;
use std::fmt;

fn main() {
    println!("Hello, world!");
    use self::mastermind::game as mastermind;
    use self::mastermind::ConfigType;
    let configuration = ConfigType {
        columns: 4,
        colors: 6,
    };
    let mut _game = mastermind::new(&configuration);
}

// mod test_package;
// fn main() {
//     test_package::test_module::test_function();
// }
