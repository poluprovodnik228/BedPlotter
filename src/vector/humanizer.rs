use super::loader::Polyline;
use egui::{pos2, Pos2};

pub struct Humanizer;

impl Humanizer {
    /// Applies subtle organic micro-variations to simulate a living human hand.
    /// `strength`: 0.0 (off) to 1.0 (full hand tremor / natural variation).
    pub fn humanize_polylines(polylines: &[Polyline], strength: f32) -> Vec<Polyline> {
        if strength <= 0.001 {
            return polylines.to_vec();
        }

        polylines
            .iter()
            .map(|poly| Self::humanize_single_polyline(poly, strength))
            .collect()
    }

    fn humanize_single_polyline(poly: &Polyline, strength: f32) -> Polyline {
        if poly.points.len() < 2 {
            return poly.clone();
        }

        // 1. Adaptive subdivision: break long straight segments into ~1.6 mm micro-segments
        let max_seg_len = 1.6;
        let mut subdivided: Vec<Pos2> = Vec::with_capacity(poly.points.len() * 2);
        subdivided.push(poly.points[0]);

        for w in poly.points.windows(2) {
            let p0 = w[0];
            let p1 = w[1];
            let d = p0.distance(p1);
            if d > max_seg_len {
                let steps = (d / max_seg_len).ceil() as usize;
                for i in 1..steps {
                    let t = i as f32 / steps as f32;
                    subdivided.push(pos2(p0.x + (p1.x - p0.x) * t, p0.y + (p1.y - p0.y) * t));
                }
            }
            subdivided.push(p1);
        }

        let amplitude = strength * 0.28; // Up to ±0.28 mm organic jitter
        let n = subdivided.len();

        let is_closed = poly.points.first() == poly.points.last();

        let humanized_pts = subdivided
            .into_iter()
            .enumerate()
            .map(|(i, pt)| {
                // Keep endpoints stable or aligned if closed
                if i == 0 || i == n - 1 {
                    return pt;
                }
                let seed_x = pt.x * 0.15;
                let seed_y = pt.y * 0.15;

                let dx = amplitude
                    * (0.65 * (seed_x * 2.3 + seed_y * 1.1).sin()
                        + 0.35 * (seed_x * 5.7 - seed_y * 2.8 + 0.7).sin());
                let dy = amplitude
                    * (0.65 * (seed_x * 2.1 - seed_y * 1.4).cos()
                        + 0.35 * (seed_x * 6.1 + seed_y * 3.2 + 1.3).cos());

                pos2(pt.x + dx, pt.y + dy)
            })
            .collect();

        let mut res = Polyline {
            points: humanized_pts,
        };
        if is_closed && res.points.len() >= 3 {
            // Guarantee closing loop
            let first = res.points[0];
            if let Some(last) = res.points.last_mut() {
                *last = first;
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanizer_anchors_endpoints() {
        let original = Polyline {
            points: vec![pos2(0.0, 0.0), pos2(10.0, 0.0)],
        };
        let humanized = Humanizer::humanize_polylines(&[original], 0.5);
        assert_eq!(humanized.len(), 1);
        let pts = &humanized[0].points;
        assert!(pts.len() > 2, "Adaptive subdivision should add points");
        assert_eq!(pts.first().unwrap(), &pos2(0.0, 0.0));
        assert_eq!(pts.last().unwrap(), &pos2(10.0, 0.0));
    }
}
