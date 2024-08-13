#[test]
fn test_position() {
    use crate::{game::Board, perft};
    use serde::{Deserialize, Serialize};
    use std::io::Read;

    #[derive(Serialize, Deserialize, Debug)]
    struct PerftTest {
        fen: String,
        depth: u8,
        nodes: u64,
    }

    let file_path = "./resources/test_perft.json";
    let mut file = std::fs::File::open(file_path).expect("Test perft not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let tests: Vec<PerftTest> = serde_json::from_str(&contents).expect("Could not parse json");

    let mut correct = 0;
    for test in &tests {
        let mut board = Board::from_fen(&test.fen).unwrap();
        let nodes = perft(test.depth, &mut board, false);

        let nodes = nodes
            .iter()
            .map(|x| x.1)
            .reduce(|acc, counter| acc + counter);

        let count = nodes.map(|n| n.count).unwrap_or(0);
        if count != test.nodes {
            println!(
                "Failed on {} depth {}, expected nodes: {} | Perft result: {}",
                test.fen, test.depth, test.nodes, count
            );
        } else {
            println!("Test passed for {}", test.fen);
            correct += 1;
        }
    }
    println!("Passed tests: {}/{}", correct, tests.len());
    assert_eq!(correct, tests.len());
}

#[test]
fn initial_position_perft() {
    use crate::{game::Board, perft};

    let results: Vec<u64> = vec![
        0,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324,
        3195901860,
        84998978956,
        2439530234167,
        69352859712417,
        2097651003696806,
        62854969297049241,
        1981066775000396238,
    ];

    let depth = 5;
    for depth in 0..=depth {
        let mut board = Board::default();
        let nodes = perft(depth, &mut board, false);

        let nodes = nodes
            .iter()
            .map(|x| x.1)
            .reduce(|acc, counter| acc + counter);

        let count = nodes.map(|n| n.count).unwrap_or(0);
        assert_eq!(count, *results.get(depth as usize).unwrap());
    }
}
