use crate::vector::loader::Polyline;

#[derive(Default)]
pub struct GcodePreview {
    pub progress: f32, // 0.0 to 1.0
    pub is_playing: bool,
}

impl GcodePreview {
    pub fn new() -> Self {
        Self {
            progress: 1.0,
            is_playing: false,
        }
    }

    pub fn total_segments(polylines: &[Polyline]) -> usize {
        polylines.iter().map(|p| p.points.len().saturating_sub(1)).sum()
    }
}
