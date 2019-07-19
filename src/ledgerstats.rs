#![warn(clippy::all)]

use graphstats::graph::Graph;
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
}
