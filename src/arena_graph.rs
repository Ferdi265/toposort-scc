use std::marker::PhantomData;
use std::ops::Index;

use id_arena::Arena;
use id_arena::ArenaBehavior;

use super::IndexGraph;
use super::Vertex;

/// An adjacency-list-based graph data structure wrapping an `Arena` from the
/// `id-arena` crate.
///
/// Stores graph vertices as lists of incoming and outgoing edges by their
/// `Id` in the graph.
#[derive(Debug, Clone)]
pub struct ArenaGraph<'a, T, A: ArenaBehavior> {
    graph: IndexGraph,
    arena_id: u32,
    phantom: PhantomData<&'a Arena<T, A>>
}

/// A builder object that allows to easily add edges to a graph
///
/// It stores a vertex index, so that edges can be added specifying only the
/// target edge or source edge.
///
/// See `ArenaGraph::from_graph()` for usage examples
#[derive(Debug)]
pub struct ArenaGraphBuilder<'g, 'a, T, A: ArenaBehavior> {
    arena_graph: &'g mut ArenaGraph<'a, T, A>,
    index: A::Id
}

impl<'a, T, A: ArenaBehavior> ArenaGraphBuilder<'_, 'a, T, A> {
    /// Returns a reference to the stored graph
    pub fn as_graph(&self) -> &ArenaGraph<'a, T, A> {
        self.arena_graph
    }

    /// Returns a mutable reference to the stored graph
    pub fn as_mut_graph(&mut self) -> &mut ArenaGraph<'a, T, A> {
        self.arena_graph
    }

    /// Returns the stored id
    pub fn index(&self) -> A::Id {
        self.index
    }

    /// Add an edge from the stored index to the passed id
    ///
    /// This method does not check for duplicate edges.
    pub fn add_out_edge(&mut self, index: A::Id) {
        self.arena_graph.graph.add_edge(A::index(self.index), A::index(index))
    }

    /// Add an edge from the passed index to the stored id
    ///
    /// This method does not check for duplicate edges.
    pub fn add_in_edge(&mut self, index: A::Id) {
        self.arena_graph.graph.add_edge(A::index(index), A::index(self.index))
    }
}

impl<'a, T, A: ArenaBehavior> ArenaGraph<'a, T, A> {
    /// Create a new graph from an existing `Arena`-based graph-like data
    /// structure
    ///
    /// The given closure will be called once for every element of `g`, with an
    /// `ArenaGraphBuilder` instance so that edges can be easily added.
    ///
    /// # Example
    ///
    /// This example creates a graph of dependencies in a hypothetical compiler
    /// or build tool, with edges from a dependency to the targets that use
    /// them.
    ///
    /// ```rust
    /// use id_arena::Arena;
    /// use id_arena::Id;
    /// use toposort_scc::ArenaGraph;
    ///
    /// // a target during compilation, having a name and dependencies
    /// struct Target { name: &'static str, deps: Vec<Id<Target>> }
    /// impl Target {
    ///     fn new(name: &'static str) -> Self {
    ///         Target { name, deps: Vec::new() }
    ///     }
    /// }
    ///
    /// let mut arena: Arena<Target> = Arena::new();
    ///
    /// let program = arena.alloc(Target::new("program"));
    /// let main_c = arena.alloc(Target::new("main.c"));
    /// let util_c = arena.alloc(Target::new("util.c"));
    /// let util_h = arena.alloc(Target::new("util.h"));
    /// let libfoo_so = arena.alloc(Target::new("libfoo_so"));
    ///
    /// arena[program].deps.extend_from_slice(&[main_c, util_c, libfoo_so]);
    /// arena[main_c].deps.push(util_h);
    /// arena[util_c].deps.push(util_h);
    ///
    /// let g = ArenaGraph::from_graph(&arena, |mut builder, target| {
    ///     for &dep in &target.deps {
    ///         builder.add_in_edge(dep);
    ///     }
    /// });
    /// ```
    ///
    /// To get a graph with edges in the other direction, use `.add_out_edge()`
    /// or the `.transpose()` method of the graph.
    pub fn from_graph<F>(g: &'a Arena<T, A>, mut f: F) -> ArenaGraph<'a, T, A>
        where F: FnMut(ArenaGraphBuilder<'_, 'a, T, A>, &T)
    {
        let mut arena_graph = ArenaGraph {
            graph: IndexGraph::with_vertices(g.len()),
            arena_id: 0,
            phantom: PhantomData
        };

        for (id, element) in g.iter() {
            arena_graph.arena_id = A::arena_id(id);

            let builder = ArenaGraphBuilder {
                arena_graph: &mut arena_graph,
                index: id,
            };

            f(builder, element);
        }

        arena_graph
    }

    /// Returns the id of the arena this graph belongs to
    pub fn arena_id(&self) -> u32 {
        self.arena_id
    }

    /// Returns a reference to the underlying `IndexGraph`
    pub fn as_index_graph(&self) -> &IndexGraph {
        &self.graph
    }

    /// Returns the underlying `IndexGraph`
    pub fn into_index_graph(self) -> IndexGraph {
        self.graph
    }

    /// Perform topological sort or find strongly connected components
    ///
    /// If the graph contains no cycles, finds the topological ordering of this
    /// graph using Kahn's algorithm and returns it as `Ok(sorted)`.
    ///
    /// If the graph contains cycles, finds the strongly connected components of
    /// this graph using Kosaraju's algorithm and returns them as `Err(cycles)`.
    ///
    /// The difference between this function and `IndexGraph::toposort_or_scc()`
    /// is that this function returns `id-arena` ids instead of indices.
    ///
    /// See `IndexGraph::toposort_or_scc()` for usage examples
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

impl<T, A: ArenaBehavior> Index<A::Id> for ArenaGraph<'_, T, A> {
    type Output = Vertex;

    fn index(&self, id: A::Id) -> &Vertex {
        &self.graph[A::index(id)]
    }
}
