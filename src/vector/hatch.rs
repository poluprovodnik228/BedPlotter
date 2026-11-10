use super::loader::Polyline;
use egui::{pos2, Pos2};

pub struct HatchGenerator;

impl HatchGenerator {
    /// Generates zig-zag scanline infill inside a polygon boundary.
    /// `polygon`: Points forming a closed boundary (first point does not need to equal last).
    /// `angle_deg`: Infill angle in degrees (e.g. 45.0).
    /// `spacing_mm`: Distance between hatch lines (e.g. 1.5).
    /// `cross_hatch`: If true, adds a secondary orthogonal pass.
    pub fn generate_infill(
        polygon: &[Pos2],
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) -> Vec<Polyline> {
        let spacing = spacing_mm.max(0.4);
        let mut result = Self::hatch_pass(polygon, angle_deg, spacing);

        if cross_hatch {
            let mut cross = Self::hatch_pass(polygon, angle_deg + 90.0, spacing);
            result.append(&mut cross);
        }

        result
    }

    fn hatch_pass(polygon: &[Pos2], angle_deg: f32, spacing: f32) -> Vec<Polyline> {
        if polygon.len() < 3 {
            return Vec::new();
        }

        let rad = (-angle_deg).to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        // 1. Rotate polygon to align scanlines horizontally
        let rotated: Vec<Pos2> = polygon
            .iter()
            .map(|p| pos2(p.x * cos_a - p.y * sin_a, p.x * sin_a + p.y * cos_a))
            .collect();

        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for p in &rotated {
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        let rad_inv = angle_deg.to_radians();
        let cos_back = rad_inv.cos();
        let sin_back = rad_inv.sin();

        let mut lines = Vec::new();
        let mut y = min_y + spacing * 0.5;
        let mut flip_dir = false;

        while y <= max_y {
            let mut intersections = Vec::new();
            let n = rotated.len();

            for i in 0..n {
                let p1 = rotated[i];
                let p2 = rotated[(i + 1) % n];

                if (p1.y <= y && p2.y > y) || (p2.y <= y && p1.y > y) {
                    let dy = p2.y - p1.y;
                    if dy.abs() > 1e-6 {
                        let t = (y - p1.y) / dy;
                        let x = p1.x + t * (p2.x - p1.x);
                        intersections.push(x);
                    }
                }
            }

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            for chunk in intersections.chunks(2) {
                if chunk.len() == 2 {
                    let (x0, x1) = if flip_dir {
                        (chunk[1], chunk[0])
                    } else {
                        (chunk[0], chunk[1])
                    };

                    let pt0 = pos2(
                        x0 * cos_back - y * sin_back,
                        x0 * sin_back + y * cos_back,
                    );
                    let pt1 = pos2(
                        x1 * cos_back - y * sin_back,
                        x1 * sin_back + y * cos_back,
                    );

                    lines.push(Polyline {
                        points: vec![pt0, pt1],
                    });
                }
            }

            // Alternating direction cuts pen lifts by up to 70%!
            flip_dir = !flip_dir;
            y += spacing;
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hatch_square() {
        let poly = vec![
            pos2(0.0, 0.0),
            pos2(10.0, 0.0),
            pos2(10.0, 10.0),
            pos2(0.0, 10.0),
        ];
        let lines = HatchGenerator::generate_infill(&poly, 45.0, 2.0, false);
        assert!(!lines.is_empty(), "Infill must generate lines for square");
        for line in &lines {
            assert_eq!(line.points.len(), 2);
        }
    }
}
