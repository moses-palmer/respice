use std::f32;
use std::fmt;
use std::iter;

/// A single point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// The X coordinate.
    pub x: f32,

    /// The Y coordinate.
    pub y: f32,
}

/// A bounded rectangle.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bounds {
    /// The top-left X coordinate.
    pub x: i32,

    /// The top-left Y coordinate.
    pub y: i32,

    /// The width.
    pub width: u32,

    /// The height.
    pub height: u32,
}

/// A transform applied to an output.
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    /// The transformation matrix.
    pub matrix: [[f32; 3]; 3],
}

impl Bounds {
    /// Returns the coordinates of the corners.
    pub fn corners(self) -> Vec<Point> {
        let x1 = self.x as f32;
        let y1 = self.y as f32;
        let x2 = x1 + self.width as f32;
        let y2 = y1 + self.height as f32;
        vec![
            Point { x: x1, y: y1 },
            Point { x: x2, y: y1 },
            Point { x: x1, y: y2 },
            Point { x: x2, y: y2 },
        ]
    }
}

impl iter::FromIterator<Point> for Bounds {
    /// Collects a sequence of points into its bounds.
    ///
    /// If the sequence is empty, an empty bounds struct is returned.
    ///
    /// # Arguments
    /// *  `iter` - The points to collect.
    fn from_iter<I: IntoIterator<Item = Point>>(iter: I) -> Self {
        // Find the maximum and minimum values
        let (top_left, bottom_right) = iter.into_iter().fold(
            (
                Point {
                    x: f32::MAX,
                    y: f32::MAX,
                },
                Point {
                    x: f32::MIN,
                    y: f32::MIN,
                },
            ),
            |(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }),
             Point { x, y }| {
                (
                    Point {
                        x: f32::min(x1, x),
                        y: f32::min(y1, y),
                    },
                    Point {
                        x: f32::max(x2, x),
                        y: f32::max(y2, y),
                    },
                )
            },
        );

        // Calculate integer values, but only if we have a valid rectangle
        if top_left.x <= bottom_right.x && top_left.y <= bottom_right.y {
            let x = top_left.x.floor() as i32;
            let y = top_left.y.floor() as i32;
            let width = (bottom_right.x - top_left.x).ceil() as u32;
            let height = (bottom_right.y - top_left.y).ceil() as u32;
            Self {
                x,
                y,
                width,
                height,
            }
        } else {
            Self::default()
        }
    }
}

impl fmt::Display for Bounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

macro_rules! mulsum {
    ($mat:expr, $v:expr) => {
        $mat.iter().enumerate().map(|(i, m)| m * $v[i]).sum::<f32>()
    };
}

impl Transform {
    /// Applies this transform to a point.
    ///
    /// # Arguments
    /// *  `point` - The point to transform.
    pub fn apply(&self, point: Point) -> Point {
        let vector = [point.x, point.y, 0.0];
        let d = mulsum!(self.matrix[2], vector);
        if d == 0.0 {
            point
        } else {
            Point {
                x: mulsum!(self.matrix[0], vector) / d,
                y: mulsum!(self.matrix[1], vector) / d,
            }
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}
