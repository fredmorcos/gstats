#![warn(clippy::all)]

use graphstats::graph::Graph;
use log::{error, info};
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(name = "input-file", help = "Input file")]
    input: String,
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
}
