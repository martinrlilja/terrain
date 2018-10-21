use delaunator;
use na;
use petgraph::stable_graph::StableGraph;
use plot::prelude::*;
use std::{f64, iter};

use {river_gen, slope_map::SlopeMap, Point2, Point3};

#[derive(Clone, Debug)]
pub struct RiverNode {
    pub pos: Point3,
}

#[derive(Clone, Debug)]
pub struct RiverEdge {
    pub flow: f64,
    pub rosgen: RiverType,
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

pub struct RiverClassifier<SM: SlopeMap> {
    slope_map: SM,
    contour: Vec<Point2>,

    pub graph: StableGraph<RiverNode, RiverEdge>,
}

impl<SM: SlopeMap> RiverClassifier<SM> {
    pub fn new(slope_map: SM, contour: Vec<Point2>) -> RiverClassifier<SM> {
        RiverClassifier {
            slope_map,
            contour,
            graph: StableGraph::new(),
        }
    }

    pub fn generate(&self, graph: &StableGraph<river_gen::RiverNode, ()>) {
        let points = {
            let mut points = iter::repeat(delaunator::Point { x: 0., y: 0. })
                .take(0 + graph.node_count())
                .collect::<Vec<_>>();

            points.splice(
                0..,
                graph.node_indices().map(|idx| {
                    let node = &graph[idx];
                    delaunator::Point {
                        x: node.pos.x,
                        y: node.pos.y,
                    }
                }),
            );

            let bb = bounding_box(self.contour.iter().cloned(), 2000.);
            //points.splice(..4, bb.iter().map(|p| delaunator::Point { x: p.x, y: p.y }));

            points
        };

        let graph_indices = graph.node_indices().collect::<Vec<_>>();

        let triangulation = delaunator::triangulate(&points).unwrap();

        let mut figure = Figure::new();
        figure.configure(Key, |k| {
            k.set(Boxed::Yes)
                .set(Position::Inside(Vertical::Top, Horizontal::Left))
        });
        //figure.configure(Axis::BottomX, |a| a.set(Range::Limits(-3000., 13000.)));
        //figure.configure(Axis::LeftY, |a| a.set(Range::Limits(-3000., 13000.)));

        let voronoi_points = triangulation
            .triangles
            .as_slice()
            .chunks(3)
            .map(|indices| {
                let a = &points[indices[0]];
                let b = &points[indices[1]];
                let c = &points[indices[2]];

                let circumcenter = circumcenter(
                    Point2::new(a.x, a.y),
                    Point2::new(b.x, b.y),
                    Point2::new(c.x, c.y),
                );

                let z = if indices.iter().any(|&i| i < 0) {
                    0.
                } else {
                    let node_a = &graph[graph_indices[indices[0] - 0]];
                    let node_b = &graph[graph_indices[indices[1] - 0]];
                    let node_c = &graph[graph_indices[indices[2] - 0]];

                    let max_z = f64::max(node_a.pos.z, f64::max(node_b.pos.z, node_c.pos.z));

                    let distance = na::distance(
                        &Point2::new(node_a.pos.x, node_a.pos.y),
                        &Point2::new(circumcenter.x, circumcenter.y),
                    );

                    max_z
                        + self
                            .slope_map
                            .sample(Point2::new(circumcenter.x, circumcenter.y))
                            * distance
                };

                figure.plot(
                    LinesPoints {
                        x: [a.x, b.x, c.x, a.x].iter(),
                        y: [a.y, b.y, c.y, a.y].iter(),
                    },
                    |lp| {
                        lp.set(Color::Black)
                            .set(LineType::Dash)
                            .set(PointSize(0.4))
                            .set(PointType::FilledCircle)
                    },
                );

                figure.plot(
                    LinesPoints {
                        x: iter::once(circumcenter.x),
                        y: iter::once(circumcenter.y),
                    },
                    |d| {
                        d.set(Color::ForestGreen)
                            .set(PointSize(1.0))
                            .set(PointType::FilledCircle)
                    },
                );

                println!(
                    "a {:?}\tb {:?}\tc {:?}\tcc {:?}",
                    a,
                    b,
                    c,
                    (circumcenter.x, circumcenter.y)
                );

                Point3::new(circumcenter.x, circumcenter.y, z)
            }).collect::<Vec<_>>();

        let areas = points
            .iter()
            .enumerate()
            .map(|(points_index, point)| {
                let hull_points = triangulation
                    .triangles
                    .iter()
                    .enumerate()
                    .filter(|(_, &i)| i == points_index)
                    .map(|(i, _)| i / 3)
                    .map(|i| voronoi_points[i])
                    .map(|p| Point2::new(p.x, p.y))
                    .collect::<Vec<_>>();

                let hull_points = graham_scan(hull_points);

                let pairs = hull_points.iter().zip(hull_points.iter().cycle().skip(1));
                for (a, b) in pairs {

                }

                let area = 0.5 * hull_points
                    .iter()
                    .zip(hull_points.iter().cycle().skip(1))
                    .map(|(&a, &b)| a.x * b.y - b.x * a.y)
                    .sum::<f64>();

                println!("{:?}", hull_points.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>());

                figure.plot(
                    LinesPoints {
                        x: hull_points
                            .iter()
                            .cycle()
                            .take(hull_points.len() + 1)
                            .map(|p| p.x),
                        y: hull_points
                            .iter()
                            .cycle()
                            .take(hull_points.len() + 1)
                            .map(|p| p.y),
                    },
                    |lp| {
                        lp.set(Color::Red)
                            .set(LineType::Dash)
                            .set(PointSize(0.4))
                            .set(PointType::FilledCircle)
                    },
                );

                //hull_points
                ()
            }).collect::<Vec<_>>();

        figure.plot(
            LinesPoints {
                x: self.contour.iter().cycle().take(self.contour.len() + 1).map(|p| p.x),
                y: self.contour.iter().cycle().take(self.contour.len() + 1).map(|p| p.y),
            },
            |lp| {
                lp.set(Color::Blue)
                    .set(LineType::Dash)
                    .set(PointSize(0.4))
                    .set(PointType::FilledCircle)
            },
        );

        figure.draw().unwrap();

        /*
        println!("points {:?}", voronoi_points);
        println!("areas {:?}", areas);
        */

        panic!();
    }
}

fn line_intersection(a: (Point2, Point2), b: (Point2, Point2)) -> Option<Point2> {
    let p1 = a.0;
    let p2 = a.1;
    let p3 = b.0;
    let p4 = b.1;

    let t_q = (p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x);
    let t_d = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if t_d <= f64::EPSILON * 2. {
        return None;
    }

    let t = t_q / t_d;

    if t >= 0. && t <= 1.0 {
        let x = p1.x + t * (p2.x - p1.x);
        let y = p1.y + t * (p2.y - p1.y);
        Some(Point2::new(x, y))
    } else {
        None
    }
}

fn circumcenter(a: Point2, b: Point2, c: Point2) -> Point2 {
    /*
    /// Tries to calculate the inverse slope of the line between two points.
    /// Unless such a line is vertical, then it tries to calculate the same,
    /// replacing `v` with `w`.
    fn try_mc((u, v): (Point2, Point2), w: Point2) -> (f64, f64) {
        if (v.y - u.y).abs() > f64::EPSILON * 2. {
            mc(u, v)
        } else {
            mc(u, w)
        }
    }

    fn mc(u: Point2, v: Point2) -> (f64, f64) {
        let m = (u.x - v.x) / (v.y - u.y);
        let x = (v.x - u.x) / 2. + u.x;
        let y = (v.y - u.y) / 2. + u.y;
        let c = y - m * x;
        (m, c)
    }

    // Find two lines perpendicular to two edges of the triangle.
    let (m1, c1) = try_mc((a, b), c);
    let (m2, c2) = try_mc((c, b), a);

    // Find the intersection. This is the center of the circumcircle of the triangle.
    let x = (c2 - c1) / (m1 - m2);
    let y = x * m1 + c1;

    Point2::new(x, y)
    */

    let d = 2. * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    Point2::new(
        d.recip()
            * ((a.x.powi(2) + a.y.powi(2)) * (b.y - c.y)
                + (b.x.powi(2) + b.y.powi(2)) * (c.y - a.y)
                + (c.x.powi(2) + c.y.powi(2)) * (a.y - b.y)),
        d.recip()
            * ((a.x.powi(2) + a.y.powi(2)) * (c.x - b.x)
                + (b.x.powi(2) + b.y.powi(2)) * (a.x - c.x)
                + (c.x.powi(2) + c.y.powi(2)) * (b.x - a.x)),
    )
}

fn bounding_box<I: Iterator<Item = Point2>>(mut points: I, margin: f64) -> [Point2; 4] {
    let first_point = points.next().unwrap();

    let mut x_min = first_point.x;
    let mut x_max = first_point.x;
    let mut y_min = first_point.y;
    let mut y_max = first_point.y;

    for p in points {
        x_min = x_min.min(p.x);
        x_max = x_max.max(p.x);
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
    }

    x_min -= margin;
    x_max += margin;
    y_min -= margin;
    y_max += margin;

    [
        Point2::new(x_min, y_min),
        Point2::new(x_max, y_min),
        Point2::new(x_min, y_max),
        Point2::new(x_max, y_max),
    ]
}

// Based on pseudo code from the Wikipedia article on Graham scan,
// which is based on the pseudo code from Introduction to algorithms.
fn graham_scan(mut points: Vec<Point2>) -> Vec<Point2> {
    if points.len() < 3 {
        return points;
    }

    let lowest_y = points[0].y;
    let (lowest_i, _lowest_y) =
        points[1..]
            .iter()
            .enumerate()
            .fold((0, lowest_y), |(lowest_i, lowest_y), (i, p)| {
                if p.y < lowest_y {
                    (i + 1, p.y)
                } else {
                    (lowest_i, lowest_y)
                }
            });

    points.swap(0, lowest_i);

    let lowest_point = points[0];

    points.sort_by(|&a, &b| {
        use std::cmp::Ordering;
        if na::distance_squared(&a, &lowest_point) < f64::EPSILON * 2. {
            Ordering::Less
        } else if na::distance_squared(&b, &lowest_point) < f64::EPSILON * 2. {
            Ordering::Greater
        } else {
            let angle_a = f64::atan2(a.y - lowest_point.y, a.x - lowest_point.x);
            let angle_b = f64::atan2(b.y - lowest_point.y, b.x - lowest_point.x);
            angle_a.partial_cmp(&angle_b).unwrap()
        }
    });

    fn ccw(p1: Point2, p2: Point2, p3: Point2) -> f64 {
        (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
    }

    let mut stack = Vec::with_capacity(points.len());
    stack.push(points[0]);
    stack.push(points[1]);
    stack.push(points[2]);

    for &p in &points[3..] {
        while stack.len() > 1
            && ccw(
                stack[if stack.len() >= 2 { stack.len() - 2 } else { 0 }],
                stack[stack.len() - 1],
                p,
            ) <= 0.
        {
            stack.pop();
        }
        stack.push(p);
    }

    stack
}

#[cfg(test)]
mod tests {
    use super::*;
    use {slope_map::ArraySlopeMap, Point2, Point3, Vector2};

    use delaunator;
    use rand::XorShiftRng;

    fn contour(contour: &[(f64, f64)], scale: f64) -> Vec<Point2> {
        contour
            .iter()
            .map(|&(x, y)| Point2::new(x, y) * scale)
            .collect()
    }

    #[test]
    fn simple_bounding_box() {
        let points = &[
            Point2::new(0., 0.),
            Point2::new(2., 2.),
            Point2::new(1., 0.),
            Point2::new(0., 2.),
            Point2::new(1., 3.),
        ];

        let bb = bounding_box(points.iter().cloned(), 0.);

        assert_eq!(bb[0], Point2::new(0., 0.));
        assert_eq!(bb[1], Point2::new(2., 0.));
        assert_eq!(bb[2], Point2::new(0., 3.));
        assert_eq!(bb[3], Point2::new(2., 3.));
    }

    #[test]
    fn simple_graham_scan() {
        let points = vec![
            Point2::new(0., 0.),
            Point2::new(2., 4.),
            Point2::new(0., 5.),
            Point2::new(-2., 4.),
        ];

        let hull = graham_scan(points.clone());

        assert_eq!(hull, points);
    }

    #[test]
    fn excluded_points_graham_scan() {
        let points = vec![
            Point2::new(0., 0.),
            Point2::new(1., 3.),
            Point2::new(2., 4.),
            Point2::new(0., 5.),
            Point2::new(-2., 4.),
        ];

        let hull = graham_scan(points);

        assert_eq!(
            hull,
            vec![
                Point2::new(0., 0.),
                Point2::new(2., 4.),
                Point2::new(0., 5.),
                Point2::new(-2., 4.),
            ]
        );
    }

    #[test]
    fn simple_circumcenter() {
        assert_eq!(
            circumcenter(
                Point2::new(0., 1.),
                Point2::new(0., 0.),
                Point2::new(1., 0.),
            ),
            Point2::new(0.5, 0.5)
        );

        assert_eq!(
            circumcenter(
                Point2::new(20., 1.),
                Point2::new(10., 0.),
                Point2::new(30., 0.),
            ),
            Point2::new(20., -49.5)
        );
    }

    #[test]
    fn classify() {
        use petgraph::stable_graph::StableGraph;
        use river_gen;

        const SCALE: f64 = 10_000.0;
        const CONTOUR: &[(f64, f64)] = &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let contour = contour(CONTOUR, SCALE);

        let mut graph = StableGraph::new();
        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(0.5, 0.0, 0.0) * SCALE,
            priority: 20,
        });
        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(0.5, 0.1, 0.0) * SCALE,
            priority: 19,
        });
        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(0.4, 0.2, 0.0) * SCALE,
            priority: 19,
        });
        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(0.6, 0.2, 0.0) * SCALE,
            priority: 19,
        });

        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(0.5, 1.0, 0.0) * SCALE,
            priority: 20,
        });
        let _node = graph.add_node(river_gen::RiverNode {
            pos: Point3::new(1.0, 0.5, 0.0) * SCALE,
            priority: 20,
        });

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let data = vec![
            0.0, 0.1, 0.1, 0.0,
            0.1, 0.2, 0.1, 0.1,
            0.0, 0.2, 0.0, 0.0,
            0.1, 0.1, 0.0, 0.0,
        ];

        let slope_map = ArraySlopeMap::new(data, 4, Vector2::new(0.0, 0.0), SCALE);

        let river_classifier = RiverClassifier::new(slope_map, contour);
        river_classifier.generate(&graph);
    }
}
