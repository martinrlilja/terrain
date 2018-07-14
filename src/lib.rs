#![feature(slice_patterns)]

extern crate alga;
extern crate nalgebra as na;
extern crate petgraph;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use na::Vector2;

use petgraph::Direction;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;

pub const EPSILON: Float = 0.001;
pub const PI: Float = std::f32::consts::PI;

pub type Float = f32;
pub type Point2 = na::Point2<Float>;
pub type Point3 = na::Point3<Float>;

#[derive(Clone, Debug)]
pub struct RiverNode {
    pub pos: Point3,
    pub priority: u32,
}

#[derive(Clone, Debug)]
pub struct RiverEdge {
    flow: Float,
    rosgen: RiverType,
}

#[derive(Clone, Debug)]
pub enum RiverType {
    Aplus,
    A,
    B,
    C,
    D,
    DA,
    E,
    F,
    G,
}

#[derive(Clone, Debug)]
pub struct RiverGenSettings {
    /// Height range in which nodes will be selected for expansion.
    pub height_range: Float,

    /// Probability of river growth.
    /// prob_growth + prob_symmetric + prob_asymetric = 1.0
    ///
    /// a(n) -> t(n) b(n)
    pub prob_growth: Float,

    /// Probability of river symmetric branching.
    /// prob_growth + prob_symmetric + prob_asymetric = 1.0
    ///
    /// a(n) -> t(n) b(n - 1) b(n - 1)
    pub prob_symmetric: Float,

    /// Probability of river asymetric branching.
    /// prob_growth + prob_symmetric + prob_asymetric = 1.0
    ///
    /// a(n) -> t(n) b(n) b(m), m < n
    pub prob_asymetric: Float,

    /// Length of a new edge.
    ///
    /// **Example value:** 2000.0
    pub edge_length: Float,

    /// Minimum distance a new node must be placed away from other edges and the contour.
    ///
    /// **Example value:** 1500 = edge_length * (3 / 4)
    pub edge_margin: Float,
}

pub struct RiverGen<Rng: rand::Rng, SM: SlopeMap> {
    rng: Rng,
    slope_map: SM,

    contour: Vec<Point2>,

    pub graph: StableGraph<RiverNode, ()>,
    candidates: Vec<NodeIndex>,
    edges: Vec<(Point2, Point2)>,

    settings: RiverGenSettings,
}

impl<Rng: rand::Rng, SM: SlopeMap> RiverGen<Rng, SM> {
    pub fn new(
        rng: Rng,
        slope_map: SM,
        contour: Vec<Point2>,
        graph: StableGraph<RiverNode, ()>,
        settings: RiverGenSettings,
    ) -> RiverGen<Rng, SM> {
        assert!(
            (settings.prob_growth + settings.prob_symmetric + settings.prob_asymetric - 1.0).abs()
                <= EPSILON,
            "prob_growth, prob_symmetric and prob_asymetric must sum up to 1.0"
        );
        assert!(settings.height_range >= 0.0);
        assert!(settings.edge_length > 0.0);
        assert!(settings.edge_margin >= 0.0);
        assert!(settings.edge_margin < settings.edge_length);

        let mut candidates = Vec::new();
        for n in graph.node_indices() {
            let has_outgoing = graph
                .neighbors_directed(n, Direction::Outgoing)
                .next()
                .is_some();
            if !has_outgoing {
                candidates.push(n);
            }
        }

        let edges = graph
            .edge_indices()
            .flat_map(|edge| {
                graph.edge_endpoints(edge).map(|(a, b)| {
                    let a = graph[a].pos;
                    let b = graph[b].pos;
                    (Point2::new(a.x, a.y), Point2::new(b.x, b.y))
                })
            })
            .collect();

        RiverGen {
            rng: rng,
            slope_map: slope_map,
            contour: contour,
            graph: graph,
            candidates: candidates,
            edges: edges,
            settings: settings,
        }
    }

    pub fn grow_network(&mut self) {
        while let Some(node_idx) = self.next_node() {
            let priority = self.graph[node_idx].priority;
            let growth_type = if priority > 1 {
                self.rng.gen::<Float>()
            } else {
                0.0
            };

            let idx = self.candidates.iter().position(|&n| n == node_idx).unwrap();
            self.candidates.swap_remove(idx);

            if growth_type - self.settings.prob_growth < 0.0 {
                // grow
                if let Some(point) = self.gen_point(node_idx) {
                    self.add_node(
                        node_idx,
                        RiverNode {
                            pos: point,
                            priority: priority,
                        },
                    );
                }
            } else if growth_type - self.settings.prob_growth - self.settings.prob_symmetric < 0.0 {
                // grow symmetric
                if let Some(point) = self.gen_point(node_idx) {
                    self.add_node(
                        node_idx,
                        RiverNode {
                            pos: point,
                            priority: priority - 1,
                        },
                    );
                }
                if let Some(point) = self.gen_point(node_idx) {
                    self.add_node(
                        node_idx,
                        RiverNode {
                            pos: point,
                            priority: priority - 1,
                        },
                    );
                }
            } else if growth_type - self.settings.prob_growth - self.settings.prob_symmetric
                - self.settings.prob_asymetric < 0.0
            {
                // grow asymetric
                let p = self.rng.gen_range(1, priority);
                if let Some(point) = self.gen_point(node_idx) {
                    self.add_node(
                        node_idx,
                        RiverNode {
                            pos: point,
                            priority: priority,
                        },
                    );
                }
                if let Some(point) = self.gen_point(node_idx) {
                    self.add_node(
                        node_idx,
                        RiverNode {
                            pos: point,
                            priority: p,
                        },
                    );
                }
            } else {
                unreachable!();
            }
        }
    }

    fn next_node(&self) -> Option<NodeIndex> {
        let lowest = self.candidates.iter().cloned().fold(None, |lowest, node| {
            let z = self.graph[node].pos.z;
            lowest.map_or(Some(z), |l| Some(z.min(l)))
        })?;

        self.candidates
            .iter()
            .cloned()
            .filter(|&node| self.graph[node].pos.z <= lowest + self.settings.height_range)
            .fold(None, |best, node| {
                best.map_or(Some(node), |b| {
                    if self.graph[b].priority > self.graph[node].priority {
                        Some(b)
                    } else {
                        Some(node)
                    }
                })
            })
    }

    fn add_node(&mut self, parent_idx: NodeIndex, node: RiverNode) -> NodeIndex {
        let pos = node.pos;
        let node_idx = self.graph.add_node(node);
        self.graph.add_edge(parent_idx, node_idx, ());

        let parent = &self.graph[parent_idx];
        self.edges.push((
            Point2::new(parent.pos.x, parent.pos.y),
            Point2::new(pos.x, pos.y),
        ));

        self.candidates.push(node_idx);

        node_idx
    }

    fn validate_point(&self, point: Point2) -> bool {
        // Try to limit the number of line segments tested by finding
        // all line segments within edge_length of parent.pos and
        // use this https://stackoverflow.com/a/1079478/1011428

        let verts = self.contour
            .iter()
            .cloned()
            .zip(self.contour.iter().cloned().cycle().skip(1));

        let contains = pnpoly(verts.clone(), point);
        let distance_contour = distance_to_point_squared(verts.clone(), point);
        let distance_edge = distance_to_point_squared(self.edges.iter().cloned(), point);

        contains
            && distance_contour
                .map(|d| d >= self.settings.edge_margin.powi(2))
                .unwrap_or(true)
            && distance_edge
                .map(|d| d >= self.settings.edge_margin.powi(2))
                .unwrap_or(true)
    }

    fn gen_point(&mut self, parent_idx: NodeIndex) -> Option<Point3> {
        use alga::linear::EuclideanSpace;
        let parent = &self.graph[parent_idx];

        for _ in 0..50 {
            let angle = self.rng.gen::<Float>() * PI * 2.0;
            let x = angle.cos() * self.settings.edge_length + parent.pos.x;
            let y = angle.sin() * self.settings.edge_length + parent.pos.y;

            if !self.validate_point(Point2::new(x, y)) {
                continue;
            }

            const SLOPE_MIN: Float = 0.001;
            const SLOPE_RANGE: Float = 0.25;
            let slope =
                self.slope_map.sample(Point2::new(x, y)) * (SLOPE_RANGE - SLOPE_MIN) + SLOPE_MIN;

            // The equations below only work in this range.
            assert!(slope > 0.0 && slope < 1.0, "slope {}", slope);

            // This is the upper limit for the new elevation according to the Lipchitz condition.
            let limit = self.settings.edge_length * (-slope.powi(2) / (slope.powi(2) - 1.0)).sqrt();
            let z = self.rng.gen::<Float>() * limit + parent.pos.z;

            let pos = Point3::new(x, y, z);

            assert!(
                pos.z >= parent.pos.z
                    && (pos.z - parent.pos.z).abs() < slope * pos.distance(&parent.pos),
                "pos {}, parent.pos {}",
                pos,
                parent.pos,
            );

            return Some(pos);
        }

        None
    }
}

/**
 * Copyright (c) 1970-2003, Wm. Randolph Franklin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * Redistributions of source code must retain the above copyright notice, this
 * list of conditions and the following disclaimers.
 * Redistributions in binary form must reproduce the above copyright notice in
 * the documentation and/or other materials provided with the distribution.
 * The name of W. Randolph Franklin may not be used to endorse or promote
 * products derived from this Software without specific prior written
 * permission.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
/// Adapted from original: https://wrf.ecse.rpi.edu//Research/Short_Notes/pnpoly.html
fn pnpoly<I>(verts: I, point: Point2) -> bool
where
    I: Iterator<Item = (Point2, Point2)>,
{
    verts.fold(false, |c, (a, b)| {
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            !c
        } else {
            c
        }
    })
}

/// From: https://stackoverflow.com/a/1501725/1011428
fn distance_to_point_squared<I>(verts: I, point: Point2) -> Option<Float>
where
    I: Iterator<Item = (Point2, Point2)>,
{
    use alga::linear::EuclideanSpace;

    verts.fold(None, |d, (a, b)| {
        let length = a.distance_squared(&b);
        let t = (0.0 as Float).max((1.0 as Float).min((point - a).dot(&(b - a)) / length));
        let projection = a + t * (b - a);
        let distance = point.distance_squared(&projection);

        d.map_or(Some(distance), |d| Some(d.min(distance)))
    })
}

pub struct ArraySlopeMap {
    data: Vec<Float>,
    size: usize,

    offset: Vector2<Float>,
    scale: Float,
}

impl ArraySlopeMap {
    pub fn new(
        data: Vec<Float>,
        size: usize,
        offset: Vector2<Float>,
        scale: Float,
    ) -> ArraySlopeMap {
        assert_eq!(size * size, data.len());
        ArraySlopeMap {
            data: data,
            size: size,
            offset: offset,
            scale: scale.recip(),
        }
    }
}

impl SlopeMap for ArraySlopeMap {
    fn sample(&self, pos: Point2) -> Float {
        let pos = (pos - self.offset) * self.scale;

        if pos.x < 0.0 || pos.x >= 1.0 || pos.y < 0.0 || pos.y >= 1.0 {
            return 0.0;
        }

        let x = (pos.x * self.size as Float) as usize;
        let y = (pos.y * self.size as Float) as usize;

        let idx = x + y * self.size;

        let val = self.data[idx];
        assert!(val >= 0.0 && val <= 1.0, "val {}", val);

        val
    }
}

pub trait SlopeMap {
    /// Valid values [0.0, 1.0]
    fn sample(&self, pos: Point2) -> Float;
}

#[cfg(test)]
mod tests {
    use ::*;

    use na;
    use rand::XorShiftRng;

    fn contour(contour: &[(Float, Float)], scale: Float) -> Vec<Point2> {
        contour
            .iter()
            .map(|&(x, y)| Point2::new(x, y) * scale)
            .collect()
    }

    fn river_generator() -> RiverGen<XorShiftRng, ArraySlopeMap> {
        use petgraph::stable_graph::StableGraph;

        const SCALE: Float = 10_000.0;
        const CONTOUR: &[(Float, Float)] = &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let contour = contour(CONTOUR, SCALE);

        let mut graph = StableGraph::new();
        let _node = graph.add_node(RiverNode {
            pos: Point3::new(0.5, 0.0, 0.0) * SCALE,
            priority: 20,
        });
        let _node = graph.add_node(RiverNode {
            pos: Point3::new(0.5, 1.0, 0.0) * SCALE,
            priority: 20,
        });
        let _node = graph.add_node(RiverNode {
            pos: Point3::new(1.0, 0.5, 0.0) * SCALE,
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
            0.1, 0.2, 0.1, 0.1,
            0.0, 0.2, 0.0, 0.0,
            0.1, 0.1, 0.0, 0.0,
        ];

        let slope_map = ArraySlopeMap::new(data, 4, na::Vector2::new(0.0, 0.0), SCALE);

        RiverGen::new(
            XorShiftRng::new_unseeded(),
            slope_map,
            contour,
            graph,
            settings,
        )
    }

    #[test]
    fn create_river_generator() {
        let _gen = river_generator();
    }

    #[test]
    fn river_generator_gen_point() {
        let mut gen = river_generator();
        let node = gen.graph.node_indices().next().unwrap();
        let point = gen.gen_point(node);

        point.expect("point is none");
    }

    #[test]
    fn river_generator_grow_network() {
        let mut gen = river_generator();
        gen.grow_network();
    }

    #[test]
    fn river_generator_validate_point() {
        let mut gen = river_generator();
        let node_a = gen.graph.node_indices().next().unwrap();
        let _node_b = gen.add_node(
            node_a,
            RiverNode {
                pos: Point3::new(5000.0, 2000.0, 10.0),
                priority: 10,
            },
        );

        // invalid points, outside
        assert!(!gen.validate_point(Point2::new(-5000.0, 0.0)));
        assert!(!gen.validate_point(Point2::new(15_000.0, 0.0)));
        assert!(!gen.validate_point(Point2::new(0.0, 15_000.0)));

        // invalid points, contour
        assert!(!gen.validate_point(Point2::new(1000.0, 1000.0)));
        assert!(!gen.validate_point(Point2::new(9_000.0, 0.0)));
        assert!(!gen.validate_point(Point2::new(0.0, 9_000.0)));

        // invalid points, edges
        assert!(!gen.validate_point(Point2::new(5000.0, -1000.0)));
        assert!(!gen.validate_point(Point2::new(5000.0, 1000.0)));
        assert!(!gen.validate_point(Point2::new(5000.0, 3000.0)));

        // valid points, contour
        assert!(gen.validate_point(Point2::new(2000.0, 2000.0)));
        assert!(gen.validate_point(Point2::new(2000.0, 2000.0)));
    }

    fn array_slope_map() -> ArraySlopeMap {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let data = vec![
            0.0, 0.1, 0.1, 0.0,
            0.1, 0.2, 0.1, 0.1,
            0.0, 0.2, 0.0, 0.0,
            0.1, 0.1, 0.0, 0.0,
        ];

        ArraySlopeMap::new(data, 4, na::Vector2::new(45.0, -10.0), 10.0)
    }

    #[test]
    fn create_array_slope_map() {
        let _map = array_slope_map();
    }

    #[test]
    fn array_slope_map_sample() {
        let map = array_slope_map();

        // points outside the map
        assert_eq!(map.sample(Point2::new(0.0, 0.0)), 0.0);
        assert_eq!(map.sample(Point2::new(60.0, 0.0)), 0.0);
        assert_eq!(map.sample(Point2::new(0.0, -20.0)), 0.0);
        assert_eq!(map.sample(Point2::new(0.0, 10.0)), 0.0);

        // points on the map
        assert_eq!(map.sample(Point2::new(45.0, -10.0)), 0.0);
        assert_eq!(map.sample(Point2::new(47.5, -10.0)), 0.1);
        assert_eq!(map.sample(Point2::new(47.5, -7.5)), 0.2);
    }
}
