# `toposort-scc`

An implementation of
[Kahn's algorithm](https://en.wikipedia.org/wiki/Topological_sorting)
for topological sorting and
[Kosaraju's algorithm](https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm)
for strongly connected components.

- an adjacency-list based graph data structure (`IndexGraph`)
- an implementation of a topological sorting algorithm that runs in
  `O(V + E)` time and `O(V)` additional space (Kahn's algorithm)
- an implementation of an algorithm that finds the strongly connected
  components of a graph in `O(V + E)` time and `O(V)` additional space
  (Kosaraju's algorithm)
- both algorithms are available via the `.toposort_or_scc()` method on
  `IndexGraph`

The `id-arena` feature adds an additional wrapper type (`ArenaGraph`) that
allows topological sorting and finding of strongly connected components on
arbitrary graph structures built with the `id-arena` crate by creating a
proxy graph that is sorted and returning a list of indices into the original
graph.

## Example

This example creates an `IndexGraph` of the example graph from the
Wikipedia page for
[Topological sorting](https://en.wikipedia.org/wiki/Topological_sorting).

A copy of the graph with cycles in it is created to demonstrate finding
of strongly connected components.

```rust
use toposort_scc::IndexGraph;

let g = IndexGraph::from_adjacency_list(&vec![
    vec![3],
    vec![3, 4],
    vec![4, 7],
    vec![5, 6, 7],
    vec![6],
    vec![],
    vec![],
    vec![]
]);

let mut g2 = g.clone();
g2.add_edge(0, 0); // trivial cycle [0]
g2.add_edge(6, 2); // cycle [2, 4, 6]

assert_eq!(g.toposort_or_scc(), Ok(vec![0, 1, 2, 3, 4, 5, 7, 6]));
assert_eq!(g2.toposort_or_scc(), Err(vec![vec![0], vec![4, 2, 6]]));
```

## Documentation

Documentation is provided via rustdoc, and can be built with `cargo doc`, or
viewed online at
[docs.rs/toposort-scc/](https://docs.rs/toposort-scc/).

## License

Licensed under either of

- Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
