mod toposort;

use toposort::Graph;

fn main() {
    let graph1 = vec![
        vec![3],
        vec![3, 4],
        vec![4],
        vec![5, 6, 7],
        vec![6],
        vec![],
        vec![],
        vec![]
    ];

    let res1 = Graph::from_graph(&graph1, |mut builder, edges| for &edge in edges {
        builder.add_out_edge(edge)
    }).toposort();

    println!("graph1: {:?}", res1);

    let graph2 = vec![
        vec![3],
        vec![3, 4],
        vec![4],
        vec![5, 6, 7],
        vec![6],
        vec![2],
        vec![2],
        vec![]
    ];

    let res2 = Graph::from_graph(&graph2, |mut builder, edges| for &edge in edges {
        builder.add_out_edge(edge)
    }).toposort();

    println!("graph2: {:?}", res2);

    let graph3 = vec![
        vec![1],
        vec![2, 4, 5],
        vec![3, 6],
        vec![2, 7],
        vec![0, 5],
        vec![6],
        vec![5],
        vec![3, 6]
    ];

    let res3 = Graph::from_graph(&graph3, |mut builder, edges| for &edge in edges {
        builder.add_out_edge(edge)
    }).toposort();

    println!("graph3: {:?}", res3);
}
