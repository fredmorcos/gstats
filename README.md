# Gstats

Gstats is a project comprising of some command-line tools and a
library for handling directed acyclic graphs with unweighted edges and
an outgoing degree of exactly 2. In addition to two outgoing edges,
vertices on the graph have a timestamp property. The two command-line
tools currently available are `gstats` and `bpdaggen`. The library is
named `graphstats`. The project is implemented using Rust.

## Building

- Debug build: `$ cargo build`
- Release build: `$ cargo build --release`

## Testing

- Testing using the debug build (slow): `$ cargo test`
- Testing using the release build (**recommended**): `$ cargo test --release`

An extension to the testing commands can be used to show standard out
and standard error from the test binaries, as in the following example
for release tests:

`$ cargo test --release -- --nocapture`

## Running

- Debug: `$ cargo run --bin TOOL_NAME -- PARAMETERS`
- Release: `$ cargo run --release --bin TOOL_NAME -- PARAMETERS`

`TOOL_NAME` can either be `gstats` or `bpdaggen`. Also, `--help` can
be passed in place of `PARAMETERS` to see the full list of
command-line arguments that each tool can accept.

The `RUST_LOG` environment variable can be used to print more
information, as in the following example for a release run of
`gstats`:

`$ RUST_LOG=gstats=info cargo run --release -- testdata/test_0.in`

## Tools

### `gstats`

`gstats` reads a graph from a file and prints statistics about
it. The graph is always assumed to have an implicitly defined root
vertex with ID 1. The graph can be bipartite but this does not seem to
be a necessity for `gstats` to function correctly. The file
format used to represent the graph is line-based and is defined as
follows:

```
N                  # The number of vertices defined in the file (excl. the root vertex)
LID RID TIMESTAMP  # LID/RID=ID of the left/right neighbour, TIMESTAMP is a timestamp
LID RID TIMESTAMP
LID RID TIMESTAMP
LID RID TIMESTAMP
...
```

Currently, `gstats` prints the following statistics:

- The average vertex depth
- The average number of transactions per depth
- The average number of incoming edges per vertex
- The average number of transactions per unit of time
- The average number of transactions per timestamp value

*The depth of a vertex is the length of the shortest path between it
and the root vertex, assuming each edge has a weight of 1.*

### `bpdaggen`

The `bpdaggen` command-line tool can be used to generate random
bipartite directed acyclic graphs of a certain size. The output graph
can be used to test the `gstats` tool. The output is printed to
standard out and can be redirected to a file. Example:

`$ cargo run --release --bin bpdaggen -- 50`
