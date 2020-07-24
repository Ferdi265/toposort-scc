// Copyright 2020 Ferdinand Bachmann
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! An implementation of
//! [Kahn's algorithm](https://en.wikipedia.org/wiki/Topological_sorting)
//! for topological sorting and
//! [Kosaraju's algorithm](https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm)
//! for strongly connected components.
//!
//! This crate provides:
//!
//! - an adjacency-list based graph data structure (`IndexGraph`)
//! - an implementation of a topological sorting algorithm that runs in
//!   `O(V + E)` time and `O(V)` additional space (Kahn's algorithm)
//! - an implementation of an algorithm that finds the strongly connected
//!   components of a graph in `O(V + E)` time and `O(V)` additional space
//!   (Kosaraju's algorithm)
//! - both algorithms are available via the `.toposort_or_scc()` method on
//!   `IndexGraph`
//!
//! The `id-arena` feature adds an additional wrapper type (`ArenaGraph`) that
//! allows topological sorting and finding of strongly connected components on
//! arbitrary graph structures built with the `id-arena` crate by creating a
//! proxy graph that is sorted and returning a list of indices into the original
//! graph.

use std::collections::VecDeque as Queue;
use std::vec::IntoIter as VecIntoIter;
use std::slice::Iter as SliceIter;
use std::ops::Index;
use std::mem;

#[cfg(feature = "id-arena")]
mod arena_graph;

#[cfg(feature = "id-arena")]
pub use arena_graph::*;

/// An adjacency-list-based graph data structure
///
/// Stores graph vertices as lists of incoming and outgoing edges by their
/// index in the graph. No additional data is stored per vertex.
#[derive(Debug, Clone)]
pub struct IndexGraph {
    vertices: Vec<Vertex>,
}

/// A vertex in an `IndexGraph`
///
/// Every vertex stores the vertices it is connected to via edges in both
/// directions.
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    in_degree: usize,
    out_degree: usize,
    pub in_edges: Vec<usize>,
    pub out_edges: Vec<usize>,
}

/// A builder object that allows to easily add edges to a graph
///
/// It stores a vertex index, so that edges can be added specifying only the
/// target edge or source edge.
///
/// See `IndexGraph::from_graph()` for usage examples
#[derive(Debug)]
pub struct IndexGraphBuilder<'g> {
    graph: &'g mut IndexGraph,
    index: usize
}

impl IndexGraphBuilder<'_> {
    /// Returns a reference to the stored graph
    pub fn as_graph(&self) -> &IndexGraph {
        self.graph
    }

    /// Returns a mutable reference to the stored graph
    pub fn as_mut_graph(&mut self) -> &mut IndexGraph {
        self.graph
    }

    /// Returns the stored index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Add an edge from the stored index to the passed index
    ///
    /// This method does not check for duplicate edges.
    pub fn add_out_edge(&mut self, index: usize) {
        self.graph.add_edge(self.index, index)
    }

    /// Add an edge from the passed index to the stored index
    ///
    /// This method does not check for duplicate edges.
    pub fn add_in_edge(&mut self, index: usize) {
        self.graph.add_edge(index, self.index)
    }
}

impl IndexGraph {
    /// Create a new graph with `len` vertices and no edges
    ///
    /// Edges can then be added with the `.add_edge()` method.
    ///
    /// # Example
    ///
    /// This example creates a graph with three vertices connected together in a
    /// cycle.
    ///
    /// ```rust
    /// use toposort_scc::IndexGraph;
    ///
    /// let mut g = IndexGraph::with_vertices(3);
    /// g.add_edge(0, 1);
    /// g.add_edge(1, 2);
    /// g.add_edge(2, 0);
    /// ```
    pub fn with_vertices(len: usize) -> Self {
        let mut vertices = Vec::with_capacity(len);
        vertices.resize_with(len, Default::default);

        IndexGraph { vertices }
    }

    /// Create a new graph from a list of adjacent vertices
    ///
    /// The graph will contain outgoing edges from each vertex to the vertices
    /// in its adjacency list.
    ///
    /// # Example
    ///
    /// This example creates an `IndexGraph` of the example graph from the
    /// Wikipedia page for
    /// [Topological sorting](https://en.wikipedia.org/wiki/Topological_sorting).
    ///
    /// ```rust
    /// use toposort_scc::IndexGraph;
    ///
    /// let g = IndexGraph::from_adjacency_list(&vec![
    ///     vec![3],
    ///     vec![3, 4],
    ///     vec![4, 7],
    ///     vec![5, 6, 7],
    ///     vec![6],
    ///     vec![],
    ///     vec![],
    ///     vec![]
    /// ]);
    /// ```
    pub fn from_adjacency_list<S>(g: &[S]) -> Self
        where S: AsRef<[usize]>
    {
        IndexGraph::from_graph(g, |mut builder, edges| for &edge in edges.as_ref() {
            builder.add_out_edge(edge)
        })
    }

    /// Create a new graph from an existing graph-like data structure
    ///
    /// The given closure will be called once for every element of `g`, with an
    /// `IndexGraphBuilder` instance so that edges can be easily added.
    ///
    /// This method is useful for creating `IndexGraphs` from existing
    /// structures.
    ///
    /// # Example
    ///
    /// This example creates a graph of dependencies in a hypothetical compiler
    /// or build tool, with edges from a dependency to the targets that use
    /// them.
    ///
    /// ```rust
    /// use toposort_scc::IndexGraph;
    ///
    /// // a target during compilation, having a name and dependencies
    /// struct Target { name: &'static str, deps: Vec<usize> }
    /// impl Target {
    ///     fn new(name: &'static str, deps: Vec<usize>) -> Self {
    ///         Target { name, deps }
    ///     }
    /// }
    ///
    /// let targets = vec![
    ///     Target::new("program", vec![1, 2, 4]),
    ///     Target::new("main.c", vec![3]),
    ///     Target::new("util.c", vec![3]),
    ///     Target::new("util.h", vec![]),
    ///     Target::new("libfoo.so", vec![])
    /// ];
    ///
    /// let g = IndexGraph::from_graph(&targets, |mut builder, target| {
    ///     for &dep in &target.deps {
    ///         builder.add_in_edge(dep);
    ///     }
    /// });
    /// ```
    ///
    /// To get a graph with edges in the other direction, use `.add_out_edge()`
    /// or the `.transpose()` method of the graph.
    ///
    /// More complicated graph structures or structures that don't store edges
    /// as indices will need to first create a `Map` to look up indices in.
    /// Alternatively, the `ArenaGraph` type from the `id-arena` feature can be
    /// used.
    pub fn from_graph<T, F>(g: &[T], mut f: F) -> Self
        where F: FnMut(IndexGraphBuilder<'_>, &T)
    {
        let mut graph = Self::with_vertices(g.len());

        for (idx, element) in g.iter().enumerate() {
            f(IndexGraphBuilder { graph: &mut graph, index: idx }, element)
        }

        graph
    }

    /// Returns an iterator over the contained vertices
    pub fn iter(&self) -> SliceIter<'_, Vertex> {
        self.vertices.iter()
    }

    /// Add a new edge to the graph
    ///
    /// This method does not check for duplicate edges.
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.vertices[from].out_degree += 1;
        self.vertices[to].in_degree += 1;
        self.vertices[from].out_edges.push(to);
        self.vertices[to].in_edges.push(from);
    }

    /// Transpose the graph
    ///
    /// Inverts the direction of all edges in the graph
    pub fn transpose(&mut self) {
        for vertex in &mut self.vertices {
            mem::swap(&mut vertex.in_degree, &mut vertex.out_degree);
            mem::swap(&mut vertex.in_edges, &mut vertex.out_edges);
        }
    }

    /// Perform topological sort or find strongly connected components
    ///
    /// If the graph contains no cycles, finds the topological ordering of this
    /// graph using Kahn's algorithm and returns it as `Ok(sorted)`.
    ///
    /// If the graph contains cycles, finds the strongly connected components of
    /// this graph using Kosaraju's algorithm and returns them as `Err(cycles)`.
    ///
    /// # Example
    ///
    /// This example creates an `IndexGraph` of the example graph from the
    /// Wikipedia page for
    /// [Topological sorting](https://en.wikipedia.org/wiki/Topological_sorting)
    /// and performs a topological sort.
    ///
    /// A copy of the graph with cycles in it is created to demonstrate finding
    /// of strongly connected components.
    ///
    /// ```rust
    /// use toposort_scc::IndexGraph;
    ///
    /// let g = IndexGraph::from_adjacency_list(&vec![
    ///     vec![3],
    ///     vec![3, 4],
    ///     vec![4, 7],
    ///     vec![5, 6, 7],
    ///     vec![6],
    ///     vec![],
    ///     vec![],
    ///     vec![]
    /// ]);
    ///
    /// let mut g2 = g.clone();
    /// g2.add_edge(0, 0); // trivial cycle [0]
    /// g2.add_edge(6, 2); // cycle [2, 4, 6]
    ///
    /// assert_eq!(g.toposort_or_scc(), Ok(vec![0, 1, 2, 3, 4, 5, 7, 6]));
    /// assert_eq!(g2.toposort_or_scc(), Err(vec![vec![0], vec![4, 2, 6]]));
    /// ```
    pub fn toposort_or_scc(mut self) -> Result<Vec<usize>, Vec<Vec<usize>>> {
        let mut queue = Queue::new();
        let mut sorted = Vec::new();

        // Kahn's algorithm for toposort

        // enqueue vertices with in-degree zero
        for (idx, vertex) in self.vertices.iter_mut().enumerate() {
            // out_degree is unused in this algorithm
            // set out_degree to zero to be used as a 'visited' flag by
            // Kosaraju's algorithm later
            vertex.out_degree = 0;

            if vertex.in_degree == 0 {
                queue.push_back(idx);
            }
        }

        // add vertices from queue to sorted list
        // decrement in-degree of neighboring edges
        // add to queue if in-degree zero
        while let Some(idx) = queue.pop_front() {
            sorted.push(idx);

            for edge_idx in 0..self.vertices[idx].out_edges.len() {
                let next_idx = self.vertices[idx].out_edges[edge_idx];

                self.vertices[next_idx].in_degree -= 1;
                if self.vertices[next_idx].in_degree == 0 {
                    queue.push_back(next_idx);
                }
            }
        }

        // if every vertex appears in sorted list, sort is successful
        if sorted.len() == self.vertices.len() {
            return Ok(sorted)
        } else {
            drop(sorted);
        }

        // else, compute strongly connected components
        // out_degree is zero everywhere, can be used as a 'visited' flag

        // Kosaraju's algorithm for strongly connected components

        // start depth-first search with first vertex
        // (empty graphs are always cycle-free, so won't reach here)
        let mut dfs_stack = vec![(0, 0)];
        self.vertices[0].out_degree = 1;

        // add vertices to queue in post-order
        while let Some((idx, edge_idx)) = dfs_stack.pop() {
            if edge_idx < self.vertices[idx].out_edges.len() {
                dfs_stack.push((idx, edge_idx + 1));

                let next_idx = self.vertices[idx].out_edges[edge_idx];
                if self.vertices[next_idx].out_degree == 0 {
                    self.vertices[next_idx].out_degree = 1;
                    dfs_stack.push((next_idx, 0));
                }
            } else {
                queue.push_back(idx);
            }
        }

        // collect cycles by depth-first search in opposite edge direction
        // from each vertex in queue
        let mut cycles = Vec::new();
        while let Some(root_idx) = queue.pop_back() {
            if self.vertices[root_idx].out_degree == 2 {
                continue
            }

            let mut cur_cycle = Vec::new();

            dfs_stack.push((root_idx, 0));

            while let Some((idx, edge_idx)) = dfs_stack.pop() {
                if edge_idx < self.vertices[idx].in_edges.len() {
                    dfs_stack.push((idx, edge_idx + 1));

                    let next_idx = self.vertices[idx].in_edges[edge_idx];
                    if self.vertices[next_idx].out_degree == 1 {
                        self.vertices[next_idx].out_degree = 2;
                        dfs_stack.push((self.vertices[idx].in_edges[edge_idx], 0));
                        cur_cycle.push(next_idx);
                    }
                }
            }

            if self.vertices[root_idx].out_degree == 2 {
                cycles.push(cur_cycle);
            } else {
                self.vertices[root_idx].out_degree = 2;
            }
        }

        // return collected cycles
        Err(cycles)
    }
}

impl Index<usize> for IndexGraph {
    type Output = Vertex;

    fn index(&self, index: usize) -> &Vertex {
        &self.vertices[index]
    }
}

impl<'g> IntoIterator for &'g IndexGraph {
    type Item = &'g Vertex;
    type IntoIter = SliceIter<'g, Vertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.iter()
    }
}

impl IntoIterator for IndexGraph {
    type Item = Vertex;
    type IntoIter = VecIntoIter<Vertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}
