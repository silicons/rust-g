use delaunator::Point;
use serde::{Deserialize, Serialize};

/**
 * This file is tightly coupled with Citadel Station's repository.
 *
 * Currently bound files:
 *
 * code/datums/math/vec2.dm
 * code/datums/math/graph.dm
 * code/datums/math/digraph.dm
 */

#[derive(Deserialize)]
#[derive(Serialize)]
struct DMVec2 {
    x: f64,
    y: f64,
}

/**
 * count is the number of vertices
 * edges are indexed, and are a list of indices an index is connected to.
 */
#[derive(Deserialize)]
#[derive(Serialize)]
struct DMGraph {
    count: usize,
    edges: Vec<Vec<usize>>,
}

impl DMGraph {
    pub fn empty_of_size(size: usize) -> DMGraph {
        let mut building = DMGraph {
            count: size,
            edges: Vec::new(),
        };
        building.edges = vec![Vec::new(); size];
        return building
    }

    pub fn connect(&mut self, a: usize, b: usize) {
        self.connect_single(a, b);
        self.connect_single(b, a);
    }

    pub fn connect_single(&mut self, a: usize, b: usize) {
        let edge_list = &mut self.edges[a];
        if edge_list.iter().any(|&e| e == b) { return; }
        edge_list.push(b);
    }
}

byond_fn!(
    fn geometry_delaunay_triangulate_to_graph(point_json) {
        let points: Vec<DMVec2> = serde_json::from_str(point_json).unwrap();
        let transmuted: Vec<Point> = points.iter().map(|p| Point{x: p.x, y: p.y}).collect();
        let triangulated = delaunator::triangulate(&transmuted);
        let mut constructing = DMGraph::empty_of_size(points.len());
        for chunk in triangulated.triangles.chunks_exact(3) {
            let a = chunk[0];
            let b = chunk[1];
            let c = chunk[2];
            constructing.connect(a.to_owned(), b.to_owned());
            constructing.connect(a.to_owned(), c.to_owned());
            constructing.connect(b.to_owned(), c.to_owned());
        };
        Some(serde_json::to_string(&constructing).unwrap())
    }
);
