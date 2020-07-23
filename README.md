# `toposort-scc`

An implementation of
[Kahn's algorithm](https://en.wikipedia.org/wiki/Topological_sorting)
for topological sorting and
[Kosaraju's algorithm](https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm)
for strongly connected components.

This crate provides:

- an adjacency-list based graph data structure
- an implementation of a topological sorting algorithm that runs in `O(V + E)`
  time and `O(V)` additional space (Kahn's algorithm)
- an implementation of an algorithm that finds the strongly connected components
  of a graph in `O(V + E)` time and `O(V)` additional space (Kosaraju's algorithm)

The `id-arena` feature adds an additional wrapper type that allows topological
sorting and finding of strongly connected components on arbitrary graph
structures built with the `id-arena` crate by creating a proxy graph that is
sorted and returning a list of indices into the original graph.

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
