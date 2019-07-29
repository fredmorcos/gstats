#![warn(clippy::all)]

mod graph;
mod id;
mod stats;
mod transaction;

use crate::graph::Graph;
use conv::ValueFrom;
use log::{error, info, warn};
use stats::Stat;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(name = "input-file", help = "Input file")]
    input: String,
}

fn main() {
    env_logger::init();

    let opts = Opt::from_args();
    info!("Input file = {}", opts.input);

    let f = match File::open(&opts.input) {
        Ok(f) => f,
        Err(e) => {
            error!("Error opening file {}: {}", opts.input, e);
            std::process::exit(1);
        }
    };

    let graph = match Graph::try_from(BufReader::new(f)) {
        Ok(g) => g,
        Err(e) => {
            error!("Error reading graph from input: {}", e);
            std::process::exit(1);
        }
    };

    info!("Loaded {} transactions", graph.len());
    info!("Graph:");
    for transaction in graph.transactions() {
        if let Some((id, transaction)) = transaction {
            info!("  {} -> {}", id, transaction);
        } else {
            error!("Invalid transaction id in graph");
            std::process::exit(1);
        }
    }

    if !graph.is_bipartite() {
        warn!("Graph is not bipartite");
    }

    let mut stats: Vec<Box<dyn Stat>> = vec![
        Box::new(stats::Depths::new(&graph)),
        Box::new(stats::InReferences::new(&graph)),
        Box::new(stats::TimeUnits::new()),
        Box::new(stats::Timestamps::new(&graph)),
    ];

    for transaction in graph.transactions() {
        if let Some((id, transaction)) = transaction {
            for stat in &mut stats {
                stat.accumulate(id, transaction);
            }
        } else {
            error!("Invalid transaction id in graph");
            std::process::exit(1);
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
