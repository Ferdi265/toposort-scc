use std::marker::PhantomData;

use id_arena::Arena;
use id_arena::ArenaBehavior;

use super::Graph;

/// An adjacency-list-based graph data structure wrapping an `Arena` from the
/// `id-arena` crate.
///
/// Stores graph vertices as lists of incoming and outgoing edges by their
/// `Id` in the graph.
#[derive(Debug, Clone)]
pub struct ArenaGraph<'a, T, A: ArenaBehavior> {
    graph: Graph,
    arena_id: u32,
    phantom: PhantomData<&'a Arena<T, A>>
}

/// A builder object that allows to easily add edges to a graph
#[derive(Debug)]
pub struct ArenaGraphBuilder<'g, 'a, T, A: ArenaBehavior> {
    graph: &'g mut Graph,
    index: usize,
    phantom: PhantomData<&'a Arena<T, A>>
}

impl<'a, T, A: ArenaBehavior> ArenaGraphBuilder<'_, 'a, T, A> {
    /// Add an edge from the stored index to the passed id
    ///
    /// This method does not check for duplicate edges.
    pub fn add_out_edge(&mut self, index: A::Id) {
        self.graph.add_edge(self.index, A::index(index))
    }

    /// Add an edge from the passed index to the stored id
    ///
    /// This method does not check for duplicate edges.
    pub fn add_in_edge(&mut self, index: A::Id) {
        self.graph.add_edge(A::index(index), self.index)
    }
}

impl<'a, T, A: ArenaBehavior> ArenaGraph<'a, T, A> {
    /// Create a new graph from an existing `Arena`-based graph-like data
    /// structure
    ///
    /// The given closure will be called once for every element of `g`, with an
    /// `ArenaGraphBuilder` instance so that edges can be easily added.
    pub fn from_graph<F>(g: &'a Arena<T, A>, mut f: F) -> ArenaGraph<'a, T, A>
        where F: FnMut(ArenaGraphBuilder<'_, 'a, T, A>, &T)
    {
        let mut arena_graph = ArenaGraph {
            graph: Graph::with_vertices(g.len()),
            arena_id: 0,
            phantom: PhantomData
        };

        for (idx, (id, element)) in g.iter().enumerate() {
            arena_graph.arena_id = A::arena_id(id);

            let builder = ArenaGraphBuilder {
                graph: &mut arena_graph.graph,
                index: idx,
                phantom: PhantomData
            };

            f(builder, element);
        }

        arena_graph
    }

    /// Perform topological sort or find strongly connected components
    ///
    /// If the graph contains no cycles, finds the topological ordering of this
    /// graph using Kahn's algorithm and returns it as `Ok(sorted)`.
    ///
    /// If the graph contains cycles, finds the strongly connected components of
    /// this graph using Kosaraju's algorithm and returns them as `Err(cycles)`.
    pub fn toposort_or_scc(self) -> Result<Vec<A::Id>, Vec<Vec<A::Id>>> {
        let arena_id = self.arena_id;

        self.graph.toposort_or_scc()
            .map(|sorted| sorted.into_iter()
                .map(|idx| A::new_id(arena_id, idx))
                .collect()
            )
            .map_err(|cycles| cycles.into_iter()
                .map(|cycle| cycle.into_iter()
                    .map(|idx| A::new_id(arena_id, idx))
                    .collect()
                )
                .collect()
            )
    }
}
