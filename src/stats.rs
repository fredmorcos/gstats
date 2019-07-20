#![warn(clippy::all)]

//! Structures that collect statistics about `Graph`s.

use crate::graph::{Graph, References};
use crate::id::Id;
use crate::id::NonRootId;
use crate::transaction::Transaction;
use conv::{errors::PosOverflow, ValueFrom};
use std::collections::{HashMap as Map, HashSet as Set};
use std::fmt::{self, Display};

/// A statistic about the graph.
pub trait Stat<'a> {
    /// Accumulate information about the graph given a transaction.
    fn accumulate(&mut self, transaction: &Transaction);

    /// When accumulation is over, this function can be called to get back a printable
    /// representation of the statistic. The errors may be caused due to invalid
    /// conversions from usizes to f64s used for divisions.
    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>>;
}

/// The result of depth statistics.
pub struct DepthsResult {
    average_depth: f64,
    average_txs_per_depth: f64,
}

impl Display for DepthsResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "> AVG DAG DEPTH: {:.2}", self.average_depth)?;
        write!(f, "> AVG TXS PER DEPTH: {:.2}", self.average_txs_per_depth)
    }
}

/// The accumulator for statistics related to transaction depths.
pub struct Depths<'a> {
    /// Keep a reference to the graph so that we can call depth().
    graph: &'a Graph,

    /// The depth calculation cache to avoid recomputing depths that have already been
    /// computed.
    cache: Map<NonRootId, usize>,

    /// The sum of all transaction depths.
    sum_of_depths: usize,

    /// The set of unique depth values.
    unique_depths: Set<usize>,
}

impl<'a> Depths<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            cache: Map::with_capacity(graph.len()),
            sum_of_depths: 0,
            unique_depths: Set::with_capacity(graph.len()),
        }
    }
}

impl<'a> Stat<'a> for Depths<'a> {
    fn accumulate(&mut self, transaction: &Transaction) {
        let depth = self.graph.depth(transaction.id(), &mut self.cache);
        self.sum_of_depths += depth;
        self.unique_depths.insert(depth);
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let n_unique_depths = f64::value_from(self.unique_depths.len())?;
        let sum_of_depths = f64::value_from(self.sum_of_depths)?;
        Ok(Box::new(DepthsResult {
            average_depth: sum_of_depths / (n_transactions + 1.0),
            average_txs_per_depth: n_transactions / n_unique_depths,
        }))
    }
}

/// The result of reverse reference statistics.
pub struct InReferencesResult {
    average_references: f64,
}

impl Display for InReferencesResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "> AVG REF: {:.2}", self.average_references)
    }
}

/// The accumulator for statistics related to reverse transaction references.
pub struct InReferences<'a> {
    /// Keep a reference to the graph to be able to access the list of reverse references.
    graph: &'a Graph,

    /// The total number of reverse references. This is an Option so that the first time
    /// we accumulate we also add the incoming reference count of the Root transaction.
    total_references: Option<usize>,
}

impl<'a> InReferences<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            total_references: None,
        }
    }
}

impl<'a> Stat<'a> for InReferences<'a> {
    fn accumulate(&mut self, transaction: &Transaction) {
        let default_refs = References::default();
        // Lookup number of references to transaction.
        let references = self
            .graph
            .references(Id::Transaction(transaction.id()))
            .unwrap_or(&default_refs);
        self.total_references = Some(self.total_references.map_or_else(
            // The accumulator is None, set it to Some() with the number of references to
            // Root + the number of references to the transaction.
            || {
                self.graph
                    .references(Id::Root)
                    .unwrap_or(&default_refs)
                    .count()
                    + references.count()
            },
            // The accumulator is a Some(), increment the accumulated value with the
            // number of references to the transaction.
            |t| t + references.count(),
        ));
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let total_references = f64::value_from(self.total_references.unwrap_or(0))?;
        Ok(Box::new(InReferencesResult {
            average_references: total_references / (n_transactions + 1.0),
        }))
    }
}

/// The result of the statistic related to time units.
pub struct TimeUnitsResult {
    average_txs_per_time_unit: f64,
}

impl Display for TimeUnitsResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "> AVG TXS PER TIME UNIT: {:.2}",
            self.average_txs_per_time_unit,
        )
    }
}

/// The accumulator for the largest timestamp.
#[derive(Default)]
pub struct TimeUnits {
    max_timestamp: usize,
}

impl Stat<'_> for TimeUnits {
    fn accumulate(&mut self, transaction: &Transaction) {
        self.max_timestamp = self.max_timestamp.max(transaction.timestamp())
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let max_timestamp = f64::value_from(self.max_timestamp)?;
        Ok(Box::new(TimeUnitsResult {
            average_txs_per_time_unit: max_timestamp / n_transactions,
        }))
    }
}

/// The result of the statistic related to timestamps.
pub struct TimestampsResult {
    average_txs_per_timestamp: f64,
}

impl Display for TimestampsResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "> AVG TXS PER TIMESTAMP: {:.2}",
            self.average_txs_per_timestamp
        )
    }
}

/// The accumulator for timestamps.
pub struct Timestamps {
    unique_timestamps: Set<usize>,
}

impl Timestamps {
    pub fn new(graph: &Graph) -> Self {
        Self {
            unique_timestamps: Set::with_capacity(graph.len()),
        }
    }
}

impl Stat<'_> for Timestamps {
    fn accumulate(&mut self, transaction: &Transaction) {
        self.unique_timestamps.insert(transaction.timestamp());
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let n_unique_timestamps = f64::value_from(self.unique_timestamps.len())?;
        Ok(Box::new(TimestampsResult {
            average_txs_per_timestamp: n_transactions / n_unique_timestamps,
        }))
    }
}
