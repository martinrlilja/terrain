extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate nalgebra as na;
extern crate petgraph;
extern crate rand;
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    macro_use
)]
extern crate stdweb;
extern crate terrain;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use stdweb::js_export;

use na::{Point2, Point3};
use petgraph::stable_graph::StableGraph;
use rand::prelude::*;
use rand::rngs::SmallRng;
use terrain::{
    river_gen::RiverGen, river_gen::RiverGenSettings, river_gen::RiverNode,
    slope_map::ArraySlopeMap,
};

// x min = 45
// x max = 640
// dx = 595
// y min = 215
// y max = 640
// dy = 425
const CONTOUR: &[(f64, f64)] = &[
    (77.142857, 363.79078),
    (134.28572, 306.64792),
    (222.85715, 283.79078),
    (282.85715, 232.3622),
    (377.14286, 215.21935),
    (442.85714, 240.93363),
    (522.85714, 220.93363),
    (591.42858, 252.36221),
    (622.85715, 318.07649),
    (597.14286, 378.07649),
    (590.0, 420.0),
    (597.14286, 455.21934),
    (640.0, 506.64792),
    (622.85715, 586.64792),
    (582.85715, 626.64792),
    (505.71429, 638.07649),
    (431.42858, 603.79078),
    (385.71429, 549.50507),
    (328.57143, 500.93364),
    (262.85715, 486.64792),
    (188.57143, 509.50506),
    (117.14286, 520.93363),
    (65.714286, 489.50506),
    (45.714286, 426.64792),
];

const SCALE: f64 = 100.0;

fn main() {}

#[derive(Deserialize)]
struct InitialRiverNodes {
    pos: (f64, f64),
    priority: u32,
}

#[derive(Deserialize)]
struct RiverGeneratorSettings {
    initial_river_nodes: Vec<InitialRiverNodes>,
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    js_export
)]
pub fn get_contour() -> Vec<f64> {
    CONTOUR
        .iter()
        .cloned()
        .flat_map(|(x, y)| vec![x * SCALE, y * SCALE])
        .collect()
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    js_export
)]
pub fn generate_river(
    prob_growth: f64,
    prob_symmetric: f64,
    prob_asymetric: f64,
    slope_map: Vec<f64>,
    contour: Vec<f64>,
) -> Vec<f64> {
    let contour = contour
        .as_slice()
        .chunks(2)
        .map(|c| Point2::new(c[0] as f64, c[1] as f64))
        .collect();

    let mut graph = StableGraph::new();
    graph.add_node(RiverNode {
        pos: Point3::new(222.85715, 283.79078, 0.0) * SCALE,
        priority: 20,
    });
    graph.add_node(RiverNode {
        pos: Point3::new(442.85714, 240.93363, 0.0) * SCALE,
        priority: 20,
    });
    graph.add_node(RiverNode {
        pos: Point3::new(590.0, 420.0, 0.0) * SCALE,
        priority: 20,
    });
    graph.add_node(RiverNode {
        pos: Point3::new(328.57143, 500.93364, 0.0) * SCALE,
        priority: 20,
    });
    graph.add_node(RiverNode {
        pos: Point3::new(188.57143, 509.50506, 0.0) * SCALE,
        priority: 20,
    });

    let settings = RiverGenSettings {
        height_range: 2.0,

        prob_growth: prob_growth as f64,
        prob_symmetric: prob_symmetric as f64,
        prob_asymetric: prob_asymetric as f64,

        edge_length: 2000.0,
        edge_margin: 1500.0,
    };

    let slope_map_size = (slope_map.len() as f64).sqrt().round() as usize;
    let slope_map = slope_map.iter().map(|&x| x as f64).collect();
    let slope_map = ArraySlopeMap::new(
        slope_map,
        slope_map_size,
        na::Vector2::new(45.0, 215.0),
        595.0 * SCALE,
    );

    let mut gen = RiverGen::new(
        SmallRng::from_entropy(),
        slope_map,
        contour,
        graph,
        settings,
    );
    gen.grow_network();

    let river_data = gen
        .graph
        .edge_indices()
        .flat_map(|edge_idx| {
            let (a_idx, b_idx) = gen.graph.edge_endpoints(edge_idx).unwrap();
            let a = &gen.graph[a_idx];
            let b = &gen.graph[b_idx];

            vec![a.pos.x, a.pos.y, a.pos.z, b.pos.x, b.pos.y, b.pos.z]
        }).collect();

    river_data
}
