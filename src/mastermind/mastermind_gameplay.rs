use crate::mastermind::mastermind_io;
//use crate::mastermind_mechanics;
use crate::mastermind::mastermind_mechanics::generate_code;
use crate::mastermind::mastermind_mechanics::CodeType;
use crate::mastermind::mastermind_mechanics::ConfigType;
use crate::mastermind::mastermind_mechanics::LineType;

pub struct game<'a> {
    guesses: Vec<LineType>,
    solution: CodeType,
    configuration: &'a ConfigType,
}

impl<'a> game<'a> {
    pub fn new(configuration: &'a ConfigType) -> Self {
        let solution = generate_code(&configuration);
        solution.print();
        let mut gameplay = Self {
            guesses: Vec::<LineType>::new(),
            solution: solution,
            configuration: &configuration,
        };
        gameplay.play();
        return gameplay;
    }
    /// get new inputs until the solution was found
    fn play(&mut self) {
        loop {
            let done = self.guess();
            if done {
                println!("!!!congratulations!!!");
                break;
            }
            self.show()
        }
    }
    /// show all previous inputs
    fn show(&self) {
        for guess in &self.guesses {
            guess.print()
        }
    }
    ///read user input and store
    fn guess(&mut self) -> bool {
        let guess = LineType::new(mastermind_io::get_code(&self.configuration), &self.solution);
        let done = guess.done(&self.configuration);
        self.guesses.push(guess);

        return done;
    }
}
