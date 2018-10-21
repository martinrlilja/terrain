use {Point2, Vector2};

pub struct ArraySlopeMap {
    data: Vec<f64>,
    size: usize,

    offset: Vector2,
    scale: f64,
}

impl ArraySlopeMap {
    pub fn new(data: Vec<f64>, size: usize, offset: Vector2, scale: f64) -> ArraySlopeMap {
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
    fn sample(&self, pos: Point2) -> f64 {
        let pos = (pos - self.offset) * self.scale;

        if pos.x < 0.0 || pos.x >= 1.0 || pos.y < 0.0 || pos.y >= 1.0 {
            return 0.0;
        }

        let x = (pos.x * self.size as f64) as usize;
        let y = (pos.y * self.size as f64) as usize;

        let idx = x + y * self.size;

        let val = self.data[idx];
        assert!(val >= 0.0 && val <= 1.0, "val {}", val);

        val
    }
}

pub trait SlopeMap {
    /// Valid values [0.0, 1.0]
    fn sample(&self, pos: Point2) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    use {Point2, Vector2};

    fn array_slope_map() -> ArraySlopeMap {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let data = vec![
            0.0, 0.1, 0.1, 0.0,
            0.1, 0.2, 0.1, 0.1,
            0.0, 0.2, 0.0, 0.0,
            0.1, 0.1, 0.0, 0.0,
        ];

        ArraySlopeMap::new(data, 4, Vector2::new(45.0, -10.0), 10.0)
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
