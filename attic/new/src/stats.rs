//! Structures that collect statistics about `Graph`s.

#![warn(clippy::all)]

use crate::graph::{Graph, InRefs};
use crate::id::Id;
use crate::id::NonRootId;
use crate::transaction::Transaction;
use conv::{errors::PosOverflow, ValueFrom};
use std::collections::{HashMap as Map, HashSet as Set};
use std::fmt::{self, Display};

pub trait Stat<'a> {
    fn accumulate(&mut self, id: NonRootId, transaction: &Transaction);
    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>>;
}

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

pub struct Depths<'a> {
    graph: &'a Graph,
    cache: Map<NonRootId, usize>,
    sum_of_depths: usize,
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
    fn accumulate(&mut self, id: NonRootId, _transaction: &Transaction) {
        let depth = self.graph.depth(id, &mut self.cache);
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

pub struct InReferencesResult {
    average_in_references: f64,
}

impl Display for InReferencesResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "> AVG REF: {:.2}", self.average_in_references)
    }
}

pub struct InReferences<'a> {
    graph: &'a Graph,
    total_in_references: Option<usize>,
}

impl<'a> InReferences<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            total_in_references: None,
        }
    }
}

impl<'a> Stat<'a> for InReferences<'a> {
    fn accumulate(&mut self, id: NonRootId, _transaction: &Transaction) {
        let default_in_refs = InRefs::default();
        let in_refs = self
            .graph
            .in_refs(Id::Transaction(id))
            .unwrap_or(&default_in_refs);
        self.total_in_references = Some(self.total_in_references.map_or_else(
            || {
                self.graph
                    .in_refs(Id::Root)
                    .unwrap_or(&default_in_refs)
                    .count()
                    + in_refs.count()
            },
            |t| t + in_refs.count(),
        ));
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let total_in_references = f64::value_from(self.total_in_references.unwrap_or(0))?;
        Ok(Box::new(InReferencesResult {
            average_in_references: total_in_references / (n_transactions + 1.0),
        }))
    }
}

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

pub struct TimeUnits {
    max_timestamp: usize,
}

impl TimeUnits {
    pub fn new() -> Self {
        Self { max_timestamp: 0 }
    }
}

impl Stat<'_> for TimeUnits {
    fn accumulate(&mut self, _id: NonRootId, transaction: &Transaction) {
        self.max_timestamp = self.max_timestamp.max(transaction.timestamp())
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let max_timestamp = f64::value_from(self.max_timestamp)?;
        Ok(Box::new(TimeUnitsResult {
            average_txs_per_time_unit: max_timestamp / n_transactions,
        }))
    }
}

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
    fn accumulate(&mut self, _id: NonRootId, transaction: &Transaction) {
        self.unique_timestamps.insert(transaction.timestamp());
    }

    fn result(&self, n_transactions: f64) -> Result<Box<dyn Display>, PosOverflow<usize>> {
        let n_unique_timestamps = f64::value_from(self.unique_timestamps.len())?;
        Ok(Box::new(TimestampsResult {
            average_txs_per_timestamp: n_transactions / n_unique_timestamps,
        }))
    }
}
