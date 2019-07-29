mod error;
mod graph;
mod id;
mod parse;
mod stats;
mod tx;
mod txref;
mod validate;

use crate::graph::Graph;
use crate::stats::Stats;
use crate::tx::Transaction;
use env_logger;
use log::{info, warn};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "ledgerstats")]
struct Opt {
    #[structopt(name = "input-file", help = "Input file")]
    input: String,

    #[structopt(short = "d", help = "Disable graph validation")]
    disable_validation: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opts = Opt::from_args();
    info!("Input file = {}", opts.input);

    let f = BufReader::new(File::open(opts.input)?);
    let graph = Graph::parse(f)?;

    if !opts.disable_validation {
        let is_bipartite = graph.validate()?;

        if !is_bipartite {
            warn!("Graph is not bipartite");
        } else {
            info!("Graph is valid");
        }
    }

    info!("Loaded {} transactions", graph.len());
    info!("Graph:");
    graph.iter().for_each(|t| info!("  {}", t));

    println!("{}", Stats::collect(&graph)?);

    Ok(())
}
