use delaunator::Point;
use serde::{Deserialize, Serialize};
use voronoice::BoundingBox;

/**
 * This file is tightly coupled with Citadel Station's repository.
 *
 * Currently bound files:
 *
 * code/datums/math/vec2.dm
 * code/datums/math/graph.dm
 * code/datums/math/digraph.dm
 */

#[derive(Serialize, Deserialize, Clone)]
struct DMVec2 {
    x: f64,
    y: f64,
    area: Option<f64>,
    cell: Option<Vec<DMVec2>>,
}

impl DMVec2 {
    pub fn polygon_area(vertices: &Vec<DMVec2>) -> f64 {
        let size = vertices.len();
        let mut area: f64 = 0_f64;
        for i in 0..size {
            let j = (i + 1) % size;
            area += vertices[i].x * vertices[j].y;
            area -= vertices[i].y * vertices[j].x;
        }
        area
    }
}

/**
 * count is the number of vertices
 * edges are indexed, and are a list of indices an index is connected to.
 */
#[derive(Serialize, Deserialize, Clone)]
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
        return building;
    }

    pub fn connect(&mut self, a: usize, b: usize) {
        self.connect_single(a, b);
        self.connect_single(b, a);
    }

    pub fn connect_single(&mut self, a: usize, b: usize) {
        let edge_list = &mut self.edges[a];
        if edge_list.iter().any(|&e| e == b) {
            return;
        }
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

/**
 * call data
 */
#[derive(Deserialize)]
struct DMDelaunayVoronoiCall {
    area: f64,
    cell: f64,
    margin: f64,
    points: Vec<DMVec2>,
}

/**
 * call return
 */
#[derive(Serialize)]
struct DMDelaunayVoronoiReturn {
    graph: DMGraph,
    areas: Vec<Option<f64>>,
    cells: Vec<Option<Vec<DMVec2>>>,
}

byond_fn!(
    fn geometry_delaunay_voronoi_graph(packed) {
        let unpacked: DMDelaunayVoronoiCall = serde_json::from_str(packed).unwrap();
        let transmuted: Vec<Point> = unpacked.points.iter().map(|p| Point{x: p.x, y: p.y}).collect();
        let mut x_low: f64 = f64::INFINITY;
        let mut x_high: f64 = -f64::INFINITY;
        let mut y_low: f64 = f64::INFINITY;
        let mut y_high: f64 = -f64::INFINITY;
        let margin = unpacked.margin;
        for point in transmuted.iter() {
            x_low = x_low.min(point.x);
            x_high = x_high.max(point.x);
            y_low = y_low.min(point.y);
            y_high = y_high.max(point.y);
        }
        let center_point = Point{x: x_low + (x_high - x_low) * 0.5, y: y_low + (y_high - y_low) * 0.5};
        let requires_area = unpacked.area != 0_f64;
        let requires_cell = unpacked.cell != 0_f64;
        let computed = voronoice::VoronoiBuilder::default()
            .set_sites(transmuted)
            .set_bounding_box(
                BoundingBox::new(center_point, (x_high - x_low) + margin * 2_f64, (y_high - y_low) + margin * 2_f64)
            )
            .build().unwrap();
        let count = unpacked.points.len();
        let mut constructing_graph = DMGraph::empty_of_size(count);
        for chunk in computed.triangulation().triangles.chunks_exact(3) {
            let a = chunk[0];
            let b = chunk[1];
            let c = chunk[2];
            constructing_graph.connect(a.to_owned(), b.to_owned());
            constructing_graph.connect(a.to_owned(), c.to_owned());
            constructing_graph.connect(b.to_owned(), c.to_owned());
        };
        let mut areas_constructed: Vec<Option<f64>> = vec![Option::None; count];
        let mut cells_constructed: Vec<Option<Vec<DMVec2>>> = vec![Option::None; count];
        for i in 0..count {
            let cell = computed.cell(i);
            let mut vertices_constructed: Vec<DMVec2> = Vec::new();
            for vertex in cell.iter_vertices() {
                vertices_constructed.push(
                    DMVec2{
                        x: vertex.x,
                        y: vertex.y,
                        area: Option::None,
                        cell: Option::None,
                    }
                );
            }
            if requires_area {
                areas_constructed[i] = Some(DMVec2::polygon_area(&vertices_constructed));
            }
            if requires_cell {
                cells_constructed[i] = Some(vertices_constructed);
            }
        }
        Some(serde_json::to_string(&DMDelaunayVoronoiReturn{
            graph: constructing_graph,
            areas: areas_constructed,
            cells: cells_constructed,
        }).unwrap())
    }
);
