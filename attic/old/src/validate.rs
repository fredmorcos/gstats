//! Validation procedures for Graphs

use crate::error::LSError;
use crate::graph::Graph;
use crate::id::Id;
use crate::txref::TxRef;
use std::collections::{HashMap as Map, HashSet as Set};

impl Graph {
    fn is_bipartite(
        vertex: TxRef,
        edges: &Map<TxRef, Set<Id>>,
        color: bool,
        colors: &mut Map<TxRef, bool>,
    ) -> bool {
        if let Some(v) = colors.get(&vertex) {
            if *v != color {
                return false;
            }
        } else {
            colors.insert(vertex, color);
        }

        if let Some(v) = edges.get(&vertex) {
            for next in v {
                if !Graph::is_bipartite(TxRef::Tx(*next), edges, !color, colors) {
                    return false;
                }
            }
        }

        true
    }

    pub fn validate(&self) -> Result<bool, LSError> {
        let mut edges: Map<TxRef, Set<Id>> = Map::with_capacity(self.len());

        let len = self.len();

        for tx in &**self {
            tx.left.is_valid(tx.id, len, LSError::LRefInvalid)?;
            tx.right.is_valid(tx.id, len, LSError::RRefInvalid)?;

            edges.entry(tx.left).or_insert_with(Set::new).insert(tx.id);
            edges.entry(tx.right).or_insert_with(Set::new).insert(tx.id);
        }

        let mut colors: Map<TxRef, bool> = Map::with_capacity(self.len());
        Ok(Graph::is_bipartite(TxRef::Root, &edges, false, &mut colors))
    }
}
