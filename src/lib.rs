#[cfg(test)]
mod tests {
    use crate::cereal_simulation::{simulation, simulation_single_thread, statistics};
    const LOOP_NUMBERS: [i32; 5] = [1, 10, 100, 1_000, 10_000];

    fn stats_wrapper(vec: Vec<i32>) {
        let (mean, median, max, min) = statistics(vec);
        println!(
            "Mean: {}, Median: {}, Max: {}, Min: {}\n",
            mean, median, max, min
        )
    }

    #[test]
    fn run_multi() {
        println!("Multi-threaded:");
        for number_of_loops in LOOP_NUMBERS.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (data, time) = simulation(*number_of_loops, 8);
            println!("Time: {:?}", time);
            stats_wrapper(data)
        }
    }

    #[test]
    fn run_single() {
        println!("Multi-threaded:");
        for number_of_loops in LOOP_NUMBERS.iter() {
            println!("Number of Simulations:{}", number_of_loops);
            let (data, time) = simulation_single_thread(*number_of_loops);
            println!("Time: {:?}", time);
            stats_wrapper(data)
        }
    }
}

pub mod cereal_simulation {

    use std::sync::mpsc::Sender;
    fn calculate(
        number_of_loops: i32,
        sender: Option<Sender<Option<Vec<i32>>>>,
    ) -> Option<Vec<i32>> {
        match number_of_loops {
            0 => {
                match sender {
                    Some(tx) => tx.send(None).unwrap(),
                    None => (),
                }
                None
            }
            _ => {
                let mut probability: Vec<i32> = Vec::new();
                // Simulation loop
                for _i in 0..number_of_loops {
                    let mut prizes: [i32; 6] = [0, 0, 0, 0, 0, 0];
                    let mut opens: i32 = 0;
                    // loop until owns every prize
                    while prizes.contains(&0) {
                        prizes[rand::thread_rng().gen_range(0, 6)] += 1;
                        opens += 1
                    }
                    probability.push(opens)
                }
                let probability = Some(probability);
                match sender {
                    Some(tx) => tx.send(probability.clone()).unwrap(),
                    None => (),
                }
                probability
            }
        }
    }

    use rand::Rng;
    use std::{
        sync, thread,
        time::{Duration, Instant},
    };
    pub fn simulation(number_of_loops: i32, number_of_threads: i32) -> (Vec<i32>, Duration) {
        if number_of_loops < 0 {
            panic!(
                "Set number of loops greater than 0. You set it to {}.",
                number_of_loops
            )
        } else if number_of_threads < 0 {
            panic!(
                "Set number of threads greater than 0. You set it to {}",
                number_of_threads
            )
        }
        // Avoid divide by zero errors with this check
        match number_of_loops {
            0 => (vec![0], Duration::new(0, 0)),
            _ => {
                // Start timer
                let start = Instant::now();
                let child_load = (number_of_loops as f32 / number_of_threads as f32).floor() as i32;
                // Create channel
                let (tx, rx) = sync::mpsc::channel();
                for _i in 0..(number_of_threads) {
                    // Create a sender and a number of loop for the other thread
                    let tx_clone = sync::mpsc::Sender::clone(&tx);
                    let child_load_clone = child_load.clone();
                    // Spawn thread for calculations
                    thread::spawn(move || calculate(child_load_clone, Some(tx_clone)));
                }
                let load = (number_of_loops as f32 / number_of_threads as f32).ceil() as i32;
                calculate(load, Some(tx));
                let mut probability = Vec::new();
                for data in rx {
                    match data {
                        Some(mut vec) => {
                            probability.append(&mut vec);
                        }
                        None => (),
                    }
                }
                probability.shrink_to_fit();
                // End timer
                let timer = start.elapsed();
                (probability, timer)
            }
        }
    }

    pub fn simulation_single_thread(number_of_loops: i32) -> (Vec<i32>, Duration) {
        // Avoid divide by zero errors with this check
        match number_of_loops {
            0 => (vec![0], Duration::new(0, 0)),
            _ => {
                // Start timer
                let start = Instant::now();
                let mut probability = calculate(number_of_loops, None).unwrap();
                probability.shrink_to_fit();
                // End timer
                let timer = start.elapsed();
                (probability, timer)
            }
        }
    }

    use statistical;
    pub fn statistics(data: Vec<i32>) -> (f64, i32, i32, i32) {
        let mut floating_data = Vec::new();
        for datum in data.clone() {
            floating_data.push(datum as f64)
        }
        let mean: f64 = statistical::mean(floating_data.as_slice());
        let median = statistical::median(data.as_slice());
        let max = data.iter().max().unwrap();
        let min = data.iter().min().unwrap();
        (mean, median, *max, *min)
    }
}
