#![feature(slice_patterns)]

extern crate alga;
extern crate delaunator;
extern crate nalgebra as na;
extern crate petgraph;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate criterion_plot as plot;

pub type Point2 = na::Point2<f64>;
pub type Point3 = na::Point3<f64>;
pub type Vector2 = na::Vector2<f64>;
pub type Vector3 = na::Vector3<f64>;

pub mod river_classifier;
pub mod river_gen;
pub mod slope_map;

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
fn distance_to_point_squared<I>(verts: I, point: Point2) -> Option<f64>
where
    I: Iterator<Item = (Point2, Point2)>,
{
    use alga::linear::EuclideanSpace;

    verts.fold(None, |d, (a, b)| {
        let length = a.distance_squared(&b);
        let t = 0.0_f64.max((1.0_f64).min((point - a).dot(&(b - a)) / length));
        let projection = a + t * (b - a);
        let distance = point.distance_squared(&projection);

        d.map_or(Some(distance), |d| Some(d.min(distance)))
    })
}
