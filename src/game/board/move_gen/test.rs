use crate::init_logging;

#[test]
fn test_position() {
    init_logging();
    use crate::{game::Board, perft};
    use serde::{Deserialize, Serialize};
    use std::io::Read;

    #[derive(Serialize, Deserialize, Debug)]
    struct PerftTest {
        fen: String,
        depth: u8,
        nodes: u64,
    }

    //parse the testposition file from /ressources/testpositions.json package root
    let file_path = "./resources/test_perft.json";
    let mut file = std::fs::File::open(file_path).expect("Test perft not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let tests: Vec<PerftTest> = serde_json::from_str(&contents).expect("Could not parse json");

    let mut correct = 0;
    for test in &tests {
        let mut board = Board::from_fen(&test.fen).unwrap();
        let nodes = perft(test.depth, &mut board);

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
        // assert_eq!(
        //     counter.count, test.nodes,
        //     "Failed on {}, expected nodes: {} | Perft result: {}",
        //     test.fen, test.nodes, counter.count
        // );
    }
    println!("Passed tests: {}/{}", correct, tests.len());
    assert_eq!(correct, tests.len());
}
