pub mod graph;
mod id;
pub mod stats;
mod transaction;

#[cfg(test)]
mod tests {
    use super::graph::Graph;
    use super::stats::{self, Stat};
    use crate::id::{Id, NonRootId};
    use crate::transaction::Transaction;
    use conv::ValueFrom;
    use std::convert::TryFrom;
    use std::io::BufReader;

    fn graph() -> Graph {
        let mut graph = Graph::default();

        graph.push(Transaction::new(
            NonRootId::try_from(2).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(1).unwrap(),
            0,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(3).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(2).unwrap(),
            0,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(4).unwrap(),
            Id::try_from(2).unwrap(),
            Id::try_from(2).unwrap(),
            1,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(5).unwrap(),
            Id::try_from(3).unwrap(),
            Id::try_from(6).unwrap(),
            3,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(6).unwrap(),
            Id::try_from(3).unwrap(),
            Id::try_from(3).unwrap(),
            2,
        ));

        graph
    }

    fn graph_from_str() -> Graph {
        let input = String::from("5\n1 1 0\n1 2 0\n2 2 1\n3 6 3\n3 3 2");
        let input = input.as_bytes();
        Graph::try_from(BufReader::new(input)).unwrap()
    }

    #[test]
    fn test() {
        let graph = graph();
        assert_eq!(graph, graph_from_str());
        assert_eq!(graph.is_connected_acyclic(), Some(true));
        assert!(!graph.is_bipartite());

        let n_transactions = f64::value_from(graph.len()).unwrap();

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

        let res = stats[0].result(n_transactions).unwrap();
        let res = format!("{}", res);
        assert_eq!(res, "> AVG DAG DEPTH: 1.33\n> AVG TXS PER DEPTH: 2.50");

        let res = stats[1].result(n_transactions).unwrap();
        let res = format!("{}", res);
        assert_eq!(res, "> AVG REF: 1.67");

        let res = stats[2].result(n_transactions).unwrap();
        let res = format!("{}", res);
        assert_eq!(res, "> AVG TXS PER TIME UNIT: 0.60");

        let res = stats[3].result(n_transactions).unwrap();
        let res = format!("{}", res);
        assert_eq!(res, "> AVG TXS PER TIMESTAMP: 1.25");
    }
}
