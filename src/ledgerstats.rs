#![warn(clippy::all)]

use conv::ValueFrom;
use graphstats::graph::Graph;
use graphstats::stats::{self, Stat};
use log::{error, info, warn};
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(name = "input-file", help = "Input file")]
    input: String,

    #[structopt(short = "-d", help = "Disable (slow) graph validation")]
    no_validation: bool,
}

// Main's return type feature could have been used, but unfortunately it means that the
// Debug implementation of errors is going to be displayed to users and this is not
// ideal. Instead, we manually handle errors in main() and use process::exit().
fn main() {
    env_logger::init();

    let opts = Opt::from_args();
    info!("Input file = {}", opts.input);

    let input_file = File::open(&opts.input).unwrap_or_else(|e| {
        error!("Error opening file `{}`: {}", opts.input, e);
        process::exit(1);
    });

    let graph = Graph::try_from(BufReader::new(input_file)).unwrap_or_else(|e| {
        error!("Error reading graph from `{}`: {}", opts.input, e);
        process::exit(2);
    });

    info!("Loaded {} transactions", graph.len());
    info!("Graph:");
    for transaction in graph.transactions() {
        info!("  {}", transaction);
    }

    if !opts.no_validation {
        match graph.is_connected_acyclic() {
            Some(true) => info!("Graph is connected and acyclic"),
            Some(false) => {
                error!("Graph is connected but cyclic, this is not supported");
                process::exit(3);
            }
            None => {
                error!("Graph is unconnected, this is not supported");
                process::exit(4);
            }
        }

        if !graph.is_bipartite() {
            warn!("Graph is not bipartite, this should not be a problem");
        } else {
            info!("Graph is bipartite");
        }
    }

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

    let n_transactions = match f64::value_from(graph.len()) {
        Ok(n) => n,
        Err(e) => {
            error!("Error converting graph length to float: {}", e);
            std::process::exit(1);
        }
    };

    for stat in stats {
        match stat.result(n_transactions) {
            Ok(r) => println!("{}", r),
            Err(e) => {
                error!("Error calculating result: {}", e);
                std::process::exit(1);
            }
        }
    }
}
