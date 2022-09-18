use conv::ValueFrom;
use graphstats::graph::Graph;
use graphstats::stats::{self, Stat};
use std::convert::TryFrom;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::BufReader;

#[test]
fn test_files() {
    for entry in fs::read_dir("testdata").unwrap() {
        let entry = entry.unwrap();
        let mut path = entry.path();

        if path.extension().unwrap() == "in" {
            let input_file = File::open(&path).unwrap();
            let graph = Graph::try_from(BufReader::new(input_file)).unwrap();
            println!("Loaded input file {:?}", path);

            assert_eq!(graph.is_connected_acyclic(), Some(true));

            let n_transactions = f64::value_from(graph.len()).unwrap();

            let mut stats: Vec<Box<dyn Stat>> = vec![
                Box::new(stats::Depths::new(&graph)),
                Box::new(stats::InReferences::new(&graph)),
                Box::new(stats::TimeUnits::default()),
                Box::new(stats::Timestamps::new(&graph)),
            ];

            for transaction in graph.transactions() {
                for stat in &mut stats {
                    stat.accumulate(transaction);
                }
            }

            let mut actual_output = String::new();

            for stat in stats {
                writeln!(
                    &mut actual_output,
                    "{}",
                    stat.result(n_transactions).unwrap()
                )
                .unwrap();
            }

            path.set_extension("out");
            let expected_output = fs::read_to_string(&path).unwrap();
            println!("Loaded expected output file {:?}", path);

            assert_eq!(actual_output, expected_output);
        }
    }
}
