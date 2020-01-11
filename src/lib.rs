#[cfg(test)]
mod tests {
    use crate::cereal_simulation::simulation;
    #[test]
    fn run_simple() {
        let loop_numbers = [1, 10, 100];
        println!("Single-threaded:");
        for number_of_loops in loop_numbers.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (max, min, mean, length) = simulation(*number_of_loops, false);
            println!(
                "Max:{} Min:{} Mean:{} Length:{:?}\n",
                max.unwrap(),
                min.unwrap(),
                mean.unwrap(),
                length.unwrap()
            );
        }
    }
    #[test]
    fn run_multi() {
        let loop_numbers = [10, 100];
        println!("Multi-threaded:");
        for number_of_loops in loop_numbers.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (max, min, mean, length) = simulation(*number_of_loops, true);
            println!(
                "Max:{} Min:{} Mean:{} Length:{:?}\n",
                max.unwrap(),
                min.unwrap(),
                mean.unwrap(),
                length.unwrap()
            );
        }
    }
}

pub mod cereal_simulation {
    use rand::{thread_rng, Rng};
    use std::sync::mpsc::Sender;
    use std::{
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    fn calculate(
        number_of_loops: i32,
        sender: Option<Sender<Option<Vec<i32>>>>,
    ) -> Option<Vec<i32>> {
        let sender_clone = sender.clone();
        let multi_core;
        match sender_clone {
            Some(_t) => {
                multi_core = true;
            }
            None => {
                multi_core = false;
            }
        }
        if number_of_loops == 0 {
            if multi_core {
                sender.unwrap().send(None).unwrap();
            }
            return None;
        }
        let mut probability: Vec<i32> = Vec::new();
        // Simulation loop
        for _i in 0..number_of_loops {
            let mut prizes: [i32; 6] = [0, 0, 0, 0, 0, 0];
            let mut opens: i32 = 0;
            // loop until owns every prize
            while prizes.contains(&0) {
                prizes[thread_rng().gen_range(0, 6)] += 1;
                opens += 1;
            }
            probability.push(opens);
        }
        if multi_core {
            sender.unwrap().send(Some(probability)).unwrap();
            None
        } else {
            Some(probability)
        }
    }

    pub fn simulation(
        number_of_loops: i32,
        concurrent: bool,
    ) -> (Option<i32>, Option<i32>, Option<f64>, Option<Duration>) {
        // Avoid divide by zero errors with this check
        if number_of_loops == 0 {
            (None, None, None, None)
        } else {
            // Start timer
            let start = Instant::now();
            // Allocate memory for variables
            let mut probability: Vec<i32>;
            if concurrent {
                // Number of threads to spawn
                const NUMBER_OF_THREADS: i32 = 4;
                let child_load = (number_of_loops as f32 / NUMBER_OF_THREADS as f32).floor() as i32;
                // Create channel
                let (tx, rx) = mpsc::channel();
                for _i in 0..NUMBER_OF_THREADS {
                    // Create a sender and a number of loop for the other thread
                    let tx = mpsc::Sender::clone(&tx);
                    let child_load_clone = child_load.clone();
                    // Spawn thread for calculations
                    thread::spawn(move || calculate(child_load_clone, Some(tx)));
                }
                let load = (number_of_loops as f32 / NUMBER_OF_THREADS as f32).ceil() as i32;
                probability = calculate(load, None).unwrap();
                for data in rx {
                    match data {
                        Some(mut T) => {
                            probability.append(&mut T);
                        }
                        None => probability.append(&mut vec![0]),
                    }
                    println!("1")
                }
            } else if !concurrent {
                probability = calculate(number_of_loops, None).unwrap();
                probability.shrink_to_fit()
            } else {
                panic!("Your computer is fucked up! Did you just data race Rust somehow?")
            }
            // End timer
            let timer = Some(start.elapsed());
            (
                Some(*probability.iter().max().unwrap()),
                Some(*probability.iter().min().unwrap()),
                Some(probability.iter().sum::<i32>() as f64 / probability.len() as f64),
                timer,
            )
        }
    }
}
