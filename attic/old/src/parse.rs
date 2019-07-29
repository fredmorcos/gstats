//! Database (Graph) and transaction parsing

use crate::error::LSError;
use crate::graph::Graph;
use crate::id::Id;
use crate::tx::Transaction;
use std::io::BufRead;
use std::str::FromStr;

impl Transaction {
    fn parse(id: Id, line: String) -> Result<Self, LSError> {
        let mut iter = line.split_ascii_whitespace();

        // Read the left reference
        let left = iter.next().ok_or(LSError::LRefMissing(id))?;
        let left = usize::from_str(&left).map_err(|e| LSError::LRefParse(id, e))?;

        // Read the right reference
        let right = iter.next().ok_or(LSError::RRefMissing(id))?;
        let right = usize::from_str(&right).map_err(|e| LSError::RRefParse(id, e))?;

        // Read the timestamp
        let timestamp = iter.next().ok_or(LSError::TSMissing(id))?;
        let timestamp = usize::from_str(&timestamp).map_err(|e| LSError::TSParse(id, e))?;

        Ok(Transaction {
            id,
            left: Id::new(left).into(),
            right: Id::new(right).into(),
            timestamp,
        })
    }
}

impl Graph {
    pub fn parse(reader: impl BufRead) -> Result<Self, LSError>
    where
        Self: Sized,
    {
        let mut iter = reader.lines();

        // Read the expected number of transactions
        let n_txs = iter.next().ok_or(LSError::NOfTxsMissing)?;
        let n_txs = usize::from_str(&n_txs?).map_err(LSError::NOfTxsInvalid)?;

        let mut res = Graph::with_capacity(n_txs);

        // Iterate over the remaining lines
        for (i, line) in iter.enumerate() {
            if i + 1 > n_txs {
                // The input exceeded the number of expected transactions
                return Err(LSError::TooManyTxs(n_txs, i + 1));
            }

            let id = Id::from(i);

            // Split the line
            let line = line?;

            // Add the transaction into the graph
            res.add_transaction(Transaction::parse(id, line)?);
        }

        // The number of transactions read was too little compared to the expected number
        if res.len() != n_txs {
            return Err(LSError::TooLittleTxs(n_txs, res.len()));
        }

        Ok(res)
    }
}
