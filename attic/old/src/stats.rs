//! Collect statistics about a graph of transactions

use crate::graph::Graph;
use crate::txref::TxRef;
use conv::{errors::PosOverflow, ValueFrom};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

/// A structure to aggregate the collected statistics
pub struct Stats {
    /// Average depth
    avg_depth: f64,
    /// Average number transactions per depth
    avg_txs_per_depth: f64,
    /// Average number of incoming references
    avg_in_references: f64,
    /// Average number of transactions per time unit
    avg_txs_per_time_unit: f64,
    /// Average number of transactions per timestamp
    avg_txs_per_ts: f64,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "> AVG DAG DEPTH: {:.2}", self.avg_depth)?;
        writeln!(f, "> AVG TXS PER DEPTH: {:.2}", self.avg_txs_per_depth)?;
        writeln!(f, "> AVG REF: {:.2}", self.avg_in_references)?;
        writeln!(
            f,
            "> AVG TXS PER TIME UNIT: {:.2}",
            self.avg_txs_per_time_unit
        )?;
        write!(f, "> AVG TXS PER TIMESTAMP: {:.2}", self.avg_txs_per_ts)
    }
}

impl Stats {
    pub fn collect(graph: &Graph) -> Result<Self, PosOverflow<usize>> {
        let mut cache = HashMap::with_capacity(graph.len());

        let mut sum_of_depths = 0.0; // Sum of depths
        let mut depths = HashSet::new(); // Unique depths
        let mut refs = HashMap::new(); // In-references per transaction
        let mut timestamps = HashSet::new(); // Unique timestamps
        let mut max_timestamp = 0;

        // For each transaction in the graph
        for tr in &**graph {
            let depth = TxRef::Tx(tr.id).depth(graph, &mut cache);

            sum_of_depths += f64::value_from(depth)?; // Accumulate the total depth
            depths.insert(depth); // Unique depths

            // Add in-references for each transaction
            *refs.entry(tr.left).or_insert(0) += 1;
            *refs.entry(tr.right).or_insert(0) += 1;

            timestamps.insert(tr.timestamp); // Unique timestamps

            if tr.timestamp > max_timestamp {
                max_timestamp = tr.timestamp
            }
        }

        let n = f64::value_from(graph.len())?; // Number of transactions (excl. the Root transaction)
        let depths_len = f64::value_from(depths.len())?; // Number of unique depths
        let timestamps_len = f64::value_from(timestamps.len())?; // Number of unique timestamps

        // The number of all in-references
        let total_in_refs: usize = refs.values().sum();
        let total_in_refs = f64::value_from(total_in_refs)?;

        let max_timestamp = f64::value_from(max_timestamp)?;

        Ok(Stats {
            // The average depth is the sum of all depth values divided by the number of
            // transactions (incl. the Root transaction)
            avg_depth: sum_of_depths / (n + 1.0),

            // The average number of transactions per depth is the number of transactions divided by
            // the number of unique depth values
            avg_txs_per_depth: n / depths_len,

            // The average number of in-references is the number of all in-references divided by the
            // number of transactions (incl. the Root transaction)
            avg_in_references: total_in_refs / (n + 1.0),

            // The average number of transactions per time unit is the value of the largest
            // timestamp divided by the number of transactions (excl. the Root transaction)
            avg_txs_per_time_unit: max_timestamp / n,

            // The average number of transactions per timestamp is the number of transactions
            // divided by the number of unique timestamps
            avg_txs_per_ts: n / timestamps_len,
        })
    }
}
