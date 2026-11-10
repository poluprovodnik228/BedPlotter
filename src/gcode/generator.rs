use crate::config::{PenMechanism, PrinterProfile};
use crate::vector::loader::Polyline;

pub struct GcodeGenerator;

impl GcodeGenerator {
    pub fn generate(polylines: &[Polyline], profile: &PrinterProfile) -> String {
        let mut out = String::new();

        // Header
        out.push_str("; BedPlotter Generated G-code\n");
        out.push_str(&profile.start_gcode);
        if !profile.start_gcode.is_empty() && !profile.start_gcode.ends_with('\n') {
            out.push('\n');
        }

        // Safe pen up before starting
        match &profile.pen_mechanism {
            PenMechanism::ZAxis { z_safe, feedrate_z, .. } => {
                out.push_str(&format!("G0 Z{:.2} F{:.0} ; pen safe\n", z_safe, feedrate_z));
            }
            PenMechanism::KlipperMacros { macro_up, .. } => {
                out.push_str(&format!("{}\n", macro_up));
            }
            PenMechanism::Servo { angle_up, pin, delay_ms, .. } => {
                out.push_str(&format!("M280 P{} S{}\nG4 P{}\n", pin, angle_up, delay_ms));
            }
        }

        for poly in polylines {
            if poly.points.is_empty() {
                continue;
            }

            // Move to first point (with toolhead offset)
            let p0 = poly.points[0];
            let x0 = p0.x - profile.toolhead_offset_x;
            let y0 = p0.y - profile.toolhead_offset_y;
            out.push_str(&format!("G0 X{:.2} Y{:.2} F{:.0}\n", x0, y0, profile.feedrate_travel));

            // Pen Down
            match &profile.pen_mechanism {
                PenMechanism::ZAxis { z_draw, feedrate_z, .. } => {
                    out.push_str(&format!("G1 Z{:.2} F{:.0} ; pen down\n", z_draw, feedrate_z));
                }
                PenMechanism::KlipperMacros { macro_down, .. } => {
                    out.push_str(&format!("{}\n", macro_down));
                }
                PenMechanism::Servo { angle_down, pin, delay_ms, .. } => {
                    out.push_str(&format!("M280 P{} S{}\nG4 P{}\n", pin, angle_down, delay_ms));
                }
            }

            // Draw segments
            for pt in &poly.points[1..] {
                let x = pt.x - profile.toolhead_offset_x;
                let y = pt.y - profile.toolhead_offset_y;
                out.push_str(&format!("G1 X{:.2} Y{:.2} F{:.0}\n", x, y, profile.feedrate_draw));
            }

            // Pen Up
            match &profile.pen_mechanism {
                PenMechanism::ZAxis { z_safe, feedrate_z, .. } => {
                    out.push_str(&format!("G0 Z{:.2} F{:.0} ; pen up\n", z_safe, feedrate_z));
                }
                PenMechanism::KlipperMacros { macro_up, .. } => {
                    out.push_str(&format!("{}\n", macro_up));
                }
                PenMechanism::Servo { angle_up, pin, delay_ms, .. } => {
                    out.push_str(&format!("M280 P{} S{}\nG4 P{}\n", pin, angle_up, delay_ms));
                }
            }
        }

        // Footer
        if !profile.end_gcode.is_empty() {
            out.push_str(&profile.end_gcode);
            if !profile.end_gcode.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }

    /// Bi-directional multipass generator: retraces lines in reverse without lifting pen
    pub fn generate_with_multipass(
        polylines: &[Polyline],
        profile: &PrinterProfile,
        multipass_count: usize,
    ) -> String {
        let count = multipass_count.max(1);
        if count == 1 {
            return Self::generate(polylines, profile);
        }

        let mut expanded = Vec::with_capacity(polylines.len());
        for poly in polylines {
            if poly.points.len() < 2 {
                continue;
            }
            let mut continuous_pts = Vec::with_capacity(poly.points.len() * count);
            for pass in 0..count {
                if pass % 2 == 0 {
                    continuous_pts.extend_from_slice(&poly.points);
                } else {
                    continuous_pts.extend(poly.points.iter().rev().cloned());
                }
            }
            expanded.push(Polyline {
                points: continuous_pts,
            });
        }

        Self::generate(&expanded, profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn test_multipass_generation() {
        let profile = PrinterProfile::anycubic_kobra_1();
        let poly = Polyline {
            points: vec![pos2(10.0, 10.0), pos2(20.0, 20.0)],
        };
        let single_pass = GcodeGenerator::generate(&[poly.clone()], &profile);
        let double_pass = GcodeGenerator::generate_with_multipass(&[poly], &profile, 2);
        assert!(double_pass.len() > single_pass.len(), "Double pass must contain more G1 moves");
    }
}
