use super::loader::Polyline;

pub struct PathOptimizer;

impl PathOptimizer {
    /// Greedy Nearest Neighbor TSP optimization for polylines to reduce travel moves.
    pub fn optimize(mut paths: Vec<Polyline>) -> Vec<Polyline> {
        if paths.is_empty() {
            return paths;
        }

        let mut result = Vec::with_capacity(paths.len());
        let mut current_pos = egui::pos2(0.0, 0.0);

        while !paths.is_empty() {
            let mut best_idx = 0;
            let mut best_dist = f32::MAX;
            let mut reverse_needed = false;

            for (idx, poly) in paths.iter().enumerate() {
                if let (Some(start), Some(end)) = (poly.points.first(), poly.points.last()) {
                    let d_start = current_pos.distance(*start);
                    let d_end = current_pos.distance(*end);

                    if d_start < best_dist {
                        best_dist = d_start;
                        best_idx = idx;
                        reverse_needed = false;
                    }
                    if d_end < best_dist {
                        best_dist = d_end;
                        best_idx = idx;
                        reverse_needed = true;
                    }
                }
            }

            let mut chosen = paths.remove(best_idx);
            if reverse_needed {
                chosen.points.reverse();
            }

            if let Some(last) = chosen.points.last() {
                current_pos = *last;
            }
            result.push(chosen);
        }

        result
    }

    /// Calculate total G0 travel distance between polylines
    pub fn travel_distance(paths: &[Polyline]) -> f32 {
        let mut total = 0.0;
        let mut cur = egui::pos2(0.0, 0.0);
        for poly in paths {
            if let Some(start) = poly.points.first() {
                total += cur.distance(*start);
            }
            if let Some(end) = poly.points.last() {
                cur = *end;
            }
        }
        total
    }
}
