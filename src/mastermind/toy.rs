// Proof of concept for concurrent brute-force method for the mastermind sovler
// basic toy problem to test the data strture and how to implement concurrency.

use arc;

mod mastermind_toy_problem {
    use core::num;
    use std::fmt;
    static N_MAX_CANDIDATES: i32 = 3;
    static N_MAX_LEVELS: i32 = 4;
    static NUMBER_STOPPER: i32 = N_MAX_CANDIDATES; // stops at max-level

    #[derive(Default, Copy, Clone, PartialEq, Debug)]
    pub enum Counter {
        #[default] // fresh will be the default
        Fresh,
        Counted(i32),
        Done(i32),
    }
    impl Counter {
        pub fn is_done(&self) -> bool {
            return match self {
                Counter::Done(_) => true,
                _ => false,
            };
        }
    }
    impl fmt::Display for Counter {
        /// implement for use in println!
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            return match *self {
                Counter::Fresh => write!(f, "Fresh"),
                Counter::Counted(num) => write!(f, "Counted ({})", num),
                Counter::Done(num) => write!(f, "DONE({})", num),
            };
        }
    }
    #[derive(Debug)]
    pub struct LevelFinder {
        level: i32,
        draw: i32,
        counter: Counter,
        candidates: Candidates,
    }

    impl LevelFinder {
        pub fn new() -> Self {
            return LevelFinder {
                level: 0,
                draw: 0,
                counter: Counter::default(),
                candidates: Candidates::default(),
            };
        }

        pub fn next(level: i32, draw: i32) -> Self {
            return LevelFinder {
                level: level,
                draw: draw,
                counter: Counter::default(),
                candidates: Candidates::default(),
            };
        }
        /// count the number of tries
        /// Count from top - down
        pub fn count(&mut self, count_previous: i32) -> &Counter {
            if self.level == N_MAX_LEVELS && self.draw == NUMBER_STOPPER {
                self.counter = Counter::Done(self.draw + count_previous);
            } else {
                let count_next = count_previous + self.draw;
                self.counter = match self.candidates.count(count_next) {
                    Counter::Fresh => Counter::Counted(count_next),
                    Counter::Counted(_) => Counter::Counted(count_next),
                    Counter::Done(num) => Counter::Done(num),
                };
            };

            return &self.counter;
        }

        /// update the status of the counter. This will be called down to the last iteration and propagate the Status Done upwards.
        pub fn count_update_status(&mut self) -> Counter {
            if let Counter::Done(num) = self.candidates.count_update_status() {
                self.counter = Counter::Done(num);
            }

            return self.counter;
        }
        /// propagate to the next level
        /// this one is executed on every level and propagates the count to the last step
        /// since propagate recursively triggers more propagations that each trigger a count
        /// therefore, the count is executed in reverse direction
        pub fn propagate(&mut self) {
            if let Counter::Done(_) = self.counter {
            } else {
                self.candidates.propagate(self.level + 1);
            }
        }

        ///check if this is the solution
        pub fn is_done(&self) -> bool {
            self.counter.is_done()
        }

        fn prune_initiate(&mut self) {
            if let Counter::Done(num) = self.counter {
                self.prune(num);
            }
        }

        /// prune all branches that obviously need more draws than a done-one
        fn prune(&mut self, num: i32) {
            self.candidates.prune(num);
        }

        pub fn show(&self) {
            let mut blanks = String::new();
            for _ in 0..self.level {
                blanks.push_str(" ");
            }
            if self.level == 0 {
                println!("####################################################################")
            }
            println!(
                "{}{} Draw: {} -> {}",
                blanks, self.level, self.draw, self.counter
            );
            self.candidates.show();
        }

        pub fn solve(&mut self) {
            for _ in 0..(N_MAX_LEVELS + 5) {
                self.propagate();
                self.count(0);
                self.count_update_status();
                self.prune_initiate();
                // self.show();

                if self.is_done() {
                    break;
                }
            }
        }
    }
    #[derive(Default, Debug)]
    enum Candidates {
        #[default]
        Fresh,
        Initialized(Vec<LevelFinder>),
    }
    impl Candidates {
        pub fn create_next_level(&mut self, level_next: i32) {
            compute();
            *self = Candidates::Initialized(
                (1..N_MAX_CANDIDATES + 1)
                    .map(|i| LevelFinder::next(level_next, i))
                    .collect(),
            )
        }
        pub fn propagate(&mut self, level_next: i32) {
            use rayon::prelude::*;
            if let Candidates::Initialized(vec) = self {
                vec.par_iter_mut().for_each(|x| x.propagate());
            } else {
                self.create_next_level(level_next);
            }
        }
        /// count the available candidates
        /// returns Done if there is one done
        /// Otherwise returns the minimum
        pub fn count(&mut self, count_previous: i32) -> Counter {
            return match self {
                Candidates::Fresh => Counter::Fresh,
                Candidates::Initialized(vec) => {
                    //return done if there is one
                    if let Some(counter) =
                        vec.iter().filter(|x| x.is_done()).next().map(|x| x.counter)
                    {
                        counter
                    } else {
                        // trigger the counter for the next level
                        vec.iter_mut().for_each(|candidate| {
                            candidate.count(count_previous);
                        });
                        //otherwise return the minimum -> cheated, always take the first
                        Counter::Counted(count_previous + 1)
                    }
                }
            };
        }
        /// after executing count (top down) execute this bottom up
        pub fn count_update_status(&self) -> Counter {
            if let Candidates::Initialized(vec) = &self {
                //return done if there is one
                if let Some(counter) = vec.iter().filter(|x| x.is_done()).next().map(|x| x.counter)
                {
                    return counter;
                } else {
                    if let Some(min) = vec
                        .iter()
                        .map(|x| {
                            if let Counter::Counted(num) = x.counter {
                                num
                            } else {
                                1000
                            }
                        })
                        .min()
                    {
                        return Counter::Counted(min);
                    } else {
                        return Counter::Fresh;
                    }
                }
            }
            Counter::Fresh
        }
        pub fn show(&self) {
            if let Candidates::Initialized(vec) = self {
                vec.iter().for_each(|x| x.show())
            }
        }
        pub fn prune(&mut self, n_draws_max: i32) {
            if let Candidates::Initialized(vec) = self {
                vec.retain(|x| match x.counter {
                    Counter::Counted(num) => num < n_draws_max,
                    Counter::Done(num) => num <= n_draws_max,
                    _ => false,
                });
                vec.iter_mut().for_each(|x| x.prune(n_draws_max))
            }
        }
    }
    /// fake the computing time
    fn compute() {
        // uncomment to cause the damage
        // use std::hint::black_box;
        // let zero = 0;
        // let max: i128 = 100000000;

        // for k in 0..max {
        //     let _c = black_box(zero) * k;
        // }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn check_counter() {
            //verification that the stopper works
            let mut lf_stop = LevelFinder::next(N_MAX_LEVELS, NUMBER_STOPPER);
            let i = 3;
            lf_stop.count(i);
            assert_eq!(lf_stop.counter, Counter::Done(NUMBER_STOPPER + i))
        }

        #[test]
        fn check_candidates() {
            let mut lf = LevelFinder::new();

            lf.solve();
            lf.show();
            let _a = 2;
            assert_eq!(lf.counter, Counter::Done(N_MAX_LEVELS + NUMBER_STOPPER - 1))
        }
        #[test]
        fn test_compute() {
            compute();
        }

        /////////////////////////////////////////////////////////////////////////////////////////
        /// Toy Problems
        #[test]
        fn test_enum() {
            // BASIC toy problem for the structure to contain the list of all candidates
            // that also has a method to propagate to the next level mutating all elements
            static N_ELEMENTS: i32 = 3;
            #[derive(Default, Debug, PartialEq)]
            enum TestEnum {
                #[default]
                Fresh,
                Initialized(Vec<i32>),
            }
            impl TestEnum {
                // see https://users.rust-lang.org/t/how-to-change-the-data-itself-via-impl-function/79526/2
                pub fn init(&mut self) {
                    *self = TestEnum::Initialized((0..N_ELEMENTS).into_iter().collect());
                }
                pub fn augment(&mut self) {
                    if let TestEnum::Initialized(vec) = self {
                        vec.iter_mut().for_each(|x| *x += 1);
                    } else {
                        self.init();
                    }
                }
            }
            let mut e = TestEnum::default();
            assert_eq!(e, TestEnum::Fresh);

            e.init();
            assert_eq!(e, TestEnum::Initialized(vec![0, 1, 2]));

            e.augment();
            assert_eq!(e, TestEnum::Initialized(vec![1, 2, 3]));
        }

        #[test]
        fn test_struct() {
            // next level -> this could work
            // elaborate toy problem for the structure to contain the list of all candidates
            // that also has a method to propagate to the next level mutating all elements
            static N_ELEMENTS: i32 = 3;
            #[derive(Debug, PartialEq)]
            struct TestStruct {
                level: i32,
                draw: i32,
                links: TestEnum,
            }
            impl TestStruct {
                pub fn new(level: i32, draw: i32) -> Self {
                    TestStruct {
                        level: level,
                        links: TestEnum::Fresh,
                        draw: draw,
                    }
                }
                pub fn propagate(&mut self) {
                    self.links.propagate(self.level + 1)
                }
            }
            #[derive(Debug, PartialEq)]
            enum TestEnum {
                Fresh,
                Initialized(Vec<TestStruct>),
            }
            impl TestEnum {
                pub fn init(&mut self, level: i32) {
                    *self = TestEnum::Initialized(
                        (0..N_ELEMENTS)
                            .into_iter()
                            .map(|x| TestStruct::new(level, x))
                            .collect(),
                    );
                }
                pub fn propagate(&mut self, level_next: i32) {
                    if let TestEnum::Initialized(vec) = self {
                        vec.iter_mut().for_each(|x| x.propagate());
                    } else {
                        self.init(level_next);
                    }
                }
            }
            let mut some_structure = TestStruct::new(0, 0);

            // create 1st level
            some_structure.propagate();
            // create 2nd level
            some_structure.propagate();
        }
        #[test]
        fn test_toy_struct() {
            struct ToyStruct {
                data: i32,
            }
            impl ToyStruct {
                pub fn augment(&mut self) {
                    self.data += 1
                }
            }

            let mut t = ToyStruct { data: 1 };

            t.augment();
            assert_eq!(t.data, 2);
        }

        #[test]
        fn test_retain() {
            #[derive(PartialEq, Debug)]
            enum test_retain {
                raw,
                initialized(Vec<i32>),
            }
            impl test_retain {
                pub fn init(&mut self) {
                    if let test_retain::raw = *self {
                        *self = test_retain::initialized((0..4).collect())
                    }
                }
                pub fn prune(&mut self) {
                    if let test_retain::initialized(vec) = self {
                        vec.retain(|x| x % 2 == 0);
                    }
                }
            }
            let mut tr: test_retain = test_retain::raw;
            tr.init();
            assert_eq!(tr, test_retain::initialized((0..4).collect()));
            tr.prune();
            assert_eq!(tr, test_retain::initialized(vec![0, 2]));
        }
    }
}
fn main() {
    use mastermind_toy_problem::LevelFinder;
    let mut lf = LevelFinder::new();

    lf.solve();
    lf.show();
}
