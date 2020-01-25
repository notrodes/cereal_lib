#[cfg(test)]
mod tests {
    use crate::cereal_simulation::simulation;
    #[test]
    fn run_simple() {
        let loop_numbers = [1, 10, 100];
        println!("Single-threaded:");
        for number_of_loops in loop_numbers.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (max, min, mean, length) = simulation(*number_of_loops, 1);
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
        let loop_numbers = [1, 10, 100];
        println!("Multi-threaded:");
        for number_of_loops in loop_numbers.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (max, min, mean, length) = simulation(*number_of_loops, 4);
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
        match number_of_loops {
            0 => {
                match sender {
                    None => sender.unwrap().send(None).unwrap(),
                    _ => (),
                }
                return None;
            }
            _ => {
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
                match sender.clone() {
                    Some(_t) => {
                        sender.unwrap().send(Some(probability)).unwrap();
                        None
                    }
                    None => Some(probability),
                }
            }
        }
    }

    pub fn simulation(
        number_of_loops: i32,
        number_of_threads: i32,
    ) -> (Option<i32>, Option<i32>, Option<f64>, Option<Duration>) {
        // Avoid divide by zero errors with this check
        match number_of_loops {
            0 => (None, None, None, None),
            _ => {
                // Start timer
                let start = Instant::now();
                let child_load = (number_of_loops as f32 / number_of_threads as f32).floor() as i32;
                // Create channel
                let (tx, rx) = mpsc::channel();
                for _i in 0..(number_of_threads - 1) {
                    // Create a sender and a number of loop for the other thread
                    let tx_clone = mpsc::Sender::clone(&tx);
                    let child_load_clone = child_load.clone();
                    // Spawn thread for calculations
                    thread::spawn(move || calculate(child_load_clone, Some(tx_clone)));
                }
                let load = (number_of_loops as f32 / number_of_threads as f32).ceil() as i32;
                let mut probability = calculate(load, None).unwrap();
                for data in rx {
                    match data {
                        Some(mut t) => {
                            probability.append(&mut t);
                        }
                        None => probability.append(&mut vec![0]),
                    }
                }
                probability.shrink_to_fit();
                // End timer
                let timer = Some(start.elapsed());
                println!("{}", *probability.iter().max().unwrap());
                (
                    Some(*probability.iter().max().unwrap()),
                    Some(*probability.iter().min().unwrap()),
                    Some(probability.iter().sum::<i32>() as f64 / probability.len() as f64),
                    timer,
                )
            }
        }
    }
}
