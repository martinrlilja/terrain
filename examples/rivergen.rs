extern crate nalgebra as na;
extern crate petgraph;
extern crate rand;
extern crate svg;
extern crate terrain;

use std::io;

use na::{Point2, Point3};
use petgraph::stable_graph::StableGraph;
use rand::prelude::*;
use rand::rngs::SmallRng;
use terrain::{
    river_gen::RiverGen, river_gen::RiverGenSettings, river_gen::RiverNode,
    slope_map::ArraySlopeMap,
};

use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

// M-18.182745,294.74779L90.913728,296.7681L175.76654000000002,246.26048L266.68027,264.44322L335.37064,321.01176L381.83765999999997,304.84932L442.44682,203.83405999999997L529.31994,163.42795999999996L519.2184,304.8493199999999L525.27932,417.9864099999999L509.11688,502.83921999999995L511.13719,581.63112L579.82756,694.7682L600.0306099999999,828.10834L519.21841,925.08299L411.13208999999995,964.47893L284.86301999999995,967.5093899999999L143.44164999999995,955.3875599999999L56.56854199999995,910.9407999999999L-24.243661000000046,854.3722999999999L-101.01525000000005,830.1286499999999L-105.05586000000005,733.1539999999999L-123.23861000000005,615.9763099999999L-171.72593000000006,472.5346499999999L-115.15739000000006,365.4584699999999Z

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

fn main() {
    const SCALE: f64 = 100.0;
    let contour = CONTOUR
        .iter()
        .map(|&(x, y)| Point2::new(x, y) * SCALE)
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

        prob_growth: 0.2,
        prob_symmetric: 0.7,
        prob_asymetric: 0.1,

        edge_length: 2000.0,
        edge_margin: 1500.0,
    };

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let data = vec![
        0.0, 0.1, 0.1, 0.0,
        0.1, 0.2, 0.3, 0.1,
        0.0, 0.1, 0.2, 0.0,
        0.1, 0.1, 0.0, 0.0,
    ];

    let slope_map = ArraySlopeMap::new(data, 4, na::Vector2::new(45.0, 215.0), 595.0 * SCALE);

    let mut gen = RiverGen::new(
        SmallRng::from_entropy(),
        slope_map,
        contour,
        graph,
        settings,
    );
    gen.grow_network();

    let river_data = gen.graph.edge_indices().fold(Data::new(), |d, edge_idx| {
        let (a_idx, b_idx) = gen.graph.edge_endpoints(edge_idx).unwrap();
        let a = &gen.graph[a_idx];
        let b = &gen.graph[b_idx];

        d.move_to((a.pos.x, a.pos.y)).line_to((b.pos.x, b.pos.y))
    });

    let river_path = Path::new()
        .set("fill", "none")
        .set("stroke", "#0a9fff")
        .set("stroke-width", 40)
        .set("d", river_data);

    let contour_data = CONTOUR
        .iter()
        .take(1)
        .fold(Data::new(), |d, &(x, y)| d.move_to((x * SCALE, y * SCALE)));
    let contour_data = CONTOUR
        .iter()
        .skip(1)
        .fold(contour_data, |d, &(x, y)| d.line_to((x * SCALE, y * SCALE)))
        .close();

    let contour_path = Path::new()
        .set("fill", "#fcfcf0")
        .set("stroke", "#6c6c4f")
        .set("stroke-width", 20)
        .set("d", contour_data);

    let document = Document::new()
        .set(
            "viewBox",
            (45.0 * SCALE, 215.0 * SCALE, 595.0 * SCALE, 425.0 * SCALE),
        ).add(contour_path)
        .add(river_path);

    svg::write(io::stdout(), &document).unwrap();

    /*
    use petgraph::dot::{Config, Dot};
    println!(
        "{:?}",
        Dot::with_config(&gen.graph, &[Config::NodeIndexLabel, Config::EdgeNoLabel])
    );
    */
}
