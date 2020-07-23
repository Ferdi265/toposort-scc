use std::collections::VecDeque as Queue;
use std::mem;

#[cfg(feature = "id-arena")]
mod arena_graph;

#[cfg(feature = "id-arena")]
pub use arena_graph::*;

#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<Vertex>,
}

#[derive(Debug, Clone, Default)]
struct Vertex {
    in_degree: usize,
    out_degree: usize,
    in_edges: Vec<usize>,
    out_edges: Vec<usize>,
}

#[derive(Debug)]
pub struct GraphBuilder<'g> {
    graph: &'g mut Graph,
    index: usize
}

impl GraphBuilder<'_> {
    pub fn add_out_edge(&mut self, index: usize) {
        self.graph.add_edge(self.index, index)
    }

    pub fn add_in_edge(&mut self, index: usize) {
        self.graph.add_edge(index, self.index)
    }
}

impl Graph {
    pub fn with_vertices(len: usize) -> Self {
        let mut vertices = Vec::with_capacity(len);
        vertices.resize_with(len, Default::default);

        Graph { vertices }
    }

    pub fn from_graph<T, F>(g: &[T], mut f: F) -> Self
        where F: FnMut(GraphBuilder<'_>, &T)
    {
        let mut graph = Self::with_vertices(g.len());

        for (idx, element) in g.iter().enumerate() {
            f(GraphBuilder { graph: &mut graph, index: idx }, element)
        }

        graph
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.vertices[from].out_degree += 1;
        self.vertices[to].in_degree += 1;
        self.vertices[from].out_edges.push(to);
        self.vertices[to].in_edges.push(from);
    }

    pub fn transpose(&mut self) {
        for vertex in &mut self.vertices {
            mem::swap(&mut vertex.in_degree, &mut vertex.out_degree);
            mem::swap(&mut vertex.in_edges, &mut vertex.out_edges);
        }
    }

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
