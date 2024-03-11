use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    #[allow(unused)]
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }
}

pub struct Rect {
    top: i32,
    bottom: i32,
    left: i32,
    right: i32,
}

fn normalize_i32_f64(x: i32, y: i32) -> (f64, f64) {
    let x = x as f64;
    let y = y as f64;
    let dist = (x * x + y * y).sqrt();
    (x / dist, y / dist)
}

pub fn compute_station_line_segment(
    prev: Option<Coord>,
    cur: Coord,
    next: Option<Coord>,
    station_length: i32,
) -> (Coord, Coord) {
    let (prev_dx, prev_dy, next_dx, next_dy) = match (prev, next) {
        (Some(prev), Some(next)) => (
            cur.x - prev.x,
            cur.y - prev.y,
            cur.x - next.x,
            cur.y - next.y,
        ),
        (Some(prev), None) => (
            cur.x - prev.x,
            cur.y - prev.y,
            prev.x - cur.x,
            prev.y - cur.y,
        ),
        (None, Some(next)) => (
            next.x - cur.x,
            next.y - cur.y,
            cur.x - next.x,
            cur.y - next.y,
        ),
        (None, None) => panic!(),
    };

    let (prev_dx_norm, prev_dy_norm) = normalize_i32_f64(prev_dx, prev_dy);
    let (next_dx_norm, next_dy_norm) = normalize_i32_f64(next_dx, next_dy);

    let dx = next_dx_norm - prev_dx_norm;
    let dy = next_dy_norm - prev_dy_norm;
    let d = (dx * dx + dy * dy).sqrt();

    let dx = (dx / d * station_length as f64 * 0.5) as i32;
    let dy = (dy / d * station_length as f64 * 0.5) as i32;

    (
        Coord::new(cur.x - dx, cur.y - dy),
        Coord::new(cur.x + dx, cur.y + dy),
    )
}

fn between(a: i32, b: i32, x: i32) -> bool {
    (a < x && x < b) || (b < x && x < a)
}

fn line_segment_cross_with_vertical_line(
    ax: i32,
    ay: i32,
    bx: i32,
    by: i32,
    x: i32,
    ylo: i32,
    yhi: i32,
) -> bool {
    if ax == bx {
        return false;
    }
    if !between(ax, bx, x) {
        return false;
    }
    // y = (x - ax) / (bx - ax) * (by - ay) + ay
    let t = (x - ax) as i64 * (by - ay) as i64 + ay as i64 * (bx - ax) as i64;
    if ax < bx {
        ylo as i64 * ((bx - ax) as i64) < t && t < yhi as i64 * (bx - ax) as i64
    } else {
        ylo as i64 * ((bx - ax) as i64) > t && t > yhi as i64 * (bx - ax) as i64
    }
}

pub fn distance_norm_square_points(p: Coord, q: Coord) -> i64 {
    let dx = (p.x - q.x) as i64;
    let dy = (p.y - q.y) as i64;
    dx * dx + dy * dy
}

pub fn distance_norm_square_point_line_segment(p: Coord, q: Coord, a: Coord) -> i64 {
    let tx = (q.x - p.x) as i64;
    let x = (p.x - a.x) as i64;
    let ty = (q.y - p.y) as i64;
    let y = (p.y - a.y) as i64;

    let a = tx * tx + ty * ty;
    let b = 2 * (tx * x + ty * y);
    let c = x * x + y * y;

    if 0 <= b {
        c
    } else if b <= -2 * a {
        a + b + c
    } else {
        let w = (b as f64 * b as f64) / (a as f64) * 0.25;
        c - w as i64
    }
}

impl Rect {
    pub fn new(top: i32, bottom: i32, left: i32, right: i32) -> Rect {
        Rect {
            top,
            bottom,
            left,
            right,
        }
    }

    #[allow(unused)]
    pub fn from_corners(a: Coord, b: Coord) -> Rect {
        Rect {
            top: a.y.min(b.y),
            bottom: a.y.max(b.y),
            left: a.x.min(b.x),
            right: a.x.max(b.x),
        }
    }

    pub fn contains(&self, pt: Coord) -> bool {
        self.top < pt.y && pt.y < self.bottom && self.left < pt.x && pt.x < self.right
    }

    pub fn crosses_with_line_segment(&self, a: Coord, b: Coord) -> bool {
        if a.x == b.x {
            return between(self.left, self.right, a.x)
                && !(a.y <= self.top && b.y <= self.top)
                && !(a.y >= self.bottom && b.y >= self.bottom);
        }

        if self.contains(a) || self.contains(b) {
            return true;
        }

        if line_segment_cross_with_vertical_line(
            a.x,
            a.y,
            b.x,
            b.y,
            self.left,
            self.top,
            self.bottom,
        ) {
            return true;
        }
        if line_segment_cross_with_vertical_line(
            a.x,
            a.y,
            b.x,
            b.y,
            self.right,
            self.top,
            self.bottom,
        ) {
            return true;
        }
        if line_segment_cross_with_vertical_line(
            a.y, a.x, b.y, b.x, self.top, self.left, self.right,
        ) {
            return true;
        }
        if line_segment_cross_with_vertical_line(
            a.y,
            a.x,
            b.y,
            b.x,
            self.bottom,
            self.left,
            self.right,
        ) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let rect = Rect::new(-1, 4, 2, 5);

        assert_eq!(rect.contains(Coord::new(3, 0)), true);
        assert_eq!(rect.contains(Coord::new(3, 5)), false);

        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(3, 0), Coord::new(1, 1)),
            true
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(4, -2), Coord::new(6, -1)),
            false
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(6, -1), Coord::new(4, -2)),
            false
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(4, -2), Coord::new(6, 1)),
            true
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(6, 1), Coord::new(4, -2)),
            true
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(0, -1), Coord::new(6, 0)),
            true
        );
        assert_eq!(
            rect.crosses_with_line_segment(Coord::new(0, -1), Coord::new(6, -2)),
            false
        );

        let rect2 = Rect::new(0, 900, 0, 1200);
        assert_eq!(
            rect2.crosses_with_line_segment(Coord::new(10906, 2266), Coord::new(11146, 2306)),
            false
        );
        assert_eq!(
            rect2.crosses_with_line_segment(Coord::new(11561, 626), Coord::new(11561, 1136)),
            false
        );
    }
}
