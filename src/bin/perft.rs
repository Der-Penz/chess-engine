use std::io::Write;

use chess_bot::{game::Board, init_logging, perft};

enum PerftMode {
    NodeCount,
    Divide,
    DivideDetailed,
    EachDepth,
}

impl From<&str> for PerftMode {
    fn from(s: &str) -> Self {
        match s {
            "nodecount" => Self::NodeCount,
            "divide" => Self::Divide,
            "dividedetailed" => Self::DivideDetailed,
            "eachdepth" => Self::EachDepth,
            _ => panic!("Invalid mode"),
        }
    }
}

fn main() {
    init_logging();
    let args: Vec<String> = std::env::args().collect();

    let depth = args
        .get(1)
        .expect("Depth must be provided")
        .parse::<u8>()
        .expect("Depth must be a number");

    let mode = args
        .get(2)
        .map(|s| PerftMode::from(s.as_str()))
        .expect("Mode must be provided (nodecount, divide, dividedetailed, eachdepth)");

    let mut board = if let Some(fen) = args.get(3) {
        Board::from_fen(fen).expect("Invalid FEN")
    } else {
        Board::default()
    };

    match mode {
        PerftMode::NodeCount => {
            let perft_result = perft(depth, &mut board);
            let node_count = perft_result
                .iter()
                .fold(0, |acc, (_, counter)| acc + counter.count);
            println!("Total nodes: {}", node_count);
        }
        PerftMode::Divide => {
            let perft_result = perft(depth, &mut board);
            for (mov, counter) in perft_result.iter() {
                println!("{}: {}", mov, counter.count);
            }
        }
        PerftMode::DivideDetailed => {
            let start = std::time::Instant::now();
            let perft_result = perft(depth, &mut board);
            let elapsed = start.elapsed();
            for (mov, counter) in perft_result.iter() {
                println!("{}: {}", mov, counter);
            }

            let total = perft_result
                .iter()
                .map(|x| x.1)
                .reduce(|acc, counter| acc + counter);
            if let Some(total) = total {
                println!();
                println!("Sum:  {}", total);
                let node_count = total.count;
                let nodes_per_second = node_count as f64 / elapsed.as_secs_f64();
                println!("Time taken: {:.3?} s", elapsed.as_secs());
                println!("Nodes per second: {:.*} N/s", 0, nodes_per_second);
                std::io::stdout().flush().unwrap();
            }
        }
        PerftMode::EachDepth => {
            for depth in 0..=depth {
                let start = std::time::Instant::now();
                let perft_result = perft(depth, &mut board);
                let elapsed = start.elapsed();
                let total = perft_result
                    .iter()
                    .map(|x| x.1)
                    .reduce(|acc, counter| acc + counter);
                if let Some(total) = total {
                    print!("Depth {}:  {}", depth, total);
                    let node_count = total.count;
                    let nodes_per_second = node_count as f64 / elapsed.as_secs_f64();
                    print!(" | Time taken: {:.3?} s", elapsed.as_secs());
                    print!(" | Nodes per second: {:.*} N/s", 0, nodes_per_second);
                    println!();
                }
            }
        }
    }
}
