use crate::config::PrinterProfile;
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Response, Stroke, Ui, Vec2};

pub struct Viewport {
    pub pan: Vec2,
    pub zoom: f32, // pixels per mm
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 2.2, // ~2.2 pixels per mm by default
        }
    }
}

impl Viewport {
    pub fn bed_to_screen(&self, bed_pt: Pos2, canvas_rect: Rect, profile: &PrinterProfile) -> Pos2 {
        let center = canvas_rect.center() + self.pan;
        let x = center.x + (bed_pt.x - profile.bed_width * 0.5) * self.zoom;
        let y = center.y - (bed_pt.y - profile.bed_height * 0.5) * self.zoom;
        pos2(x, y)
    }

    pub fn screen_to_bed(&self, screen_pt: Pos2, canvas_rect: Rect, profile: &PrinterProfile) -> Pos2 {
        let center = canvas_rect.center() + self.pan;
        let bed_x = (screen_pt.x - center.x) / self.zoom + profile.bed_width * 0.5;
        let bed_y = profile.bed_height * 0.5 - (screen_pt.y - center.y) / self.zoom;
        pos2(bed_x, bed_y)
    }

    pub fn handle_input(&mut self, ui: &Ui, response: &Response) {
        // Pan: Middle mouse drag or Right mouse drag
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            self.pan += response.drag_delta();
        }

        // Zoom: Mouse wheel
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 && response.hovered() {
            let pointer_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(response.rect.center());
            let factor = if scroll > 0.0 { 1.12 } else { 1.0 / 1.12 };
            let new_zoom = (self.zoom * factor).clamp(0.4, 25.0);

            // Zoom centered on pointer
            let canvas_center = response.rect.center() + self.pan;
            let offset_from_center = pointer_pos - canvas_center;
            self.pan -= offset_from_center * (factor - 1.0);
            self.zoom = new_zoom;
        }
    }

    pub fn reset_view(&mut self, canvas_rect: Rect, profile: &PrinterProfile) {
        self.pan = Vec2::ZERO;
        let margin = 50.0;
        let scale_x = (canvas_rect.width() - margin * 2.0) / profile.bed_width;
        let scale_y = (canvas_rect.height() - margin * 2.0) / profile.bed_height;
        self.zoom = scale_x.min(scale_y).clamp(0.5, 10.0);
    }

    pub fn render_rectified_photo(
        &self,
        painter: &Painter,
        canvas_rect: Rect,
        profile: &PrinterProfile,
        texture: &egui::TextureHandle,
        opacity: f32,
    ) {
        let p_bl = self.bed_to_screen(pos2(0.0, 0.0), canvas_rect, profile);
        let p_tr = self.bed_to_screen(pos2(profile.bed_width, profile.bed_height), canvas_rect, profile);
        let bed_screen_rect = Rect::from_two_pos(p_tr, p_bl);

        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let tint = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

        painter.image(
            texture.id(),
            bed_screen_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            tint,
        );
    }

    pub fn render_vector(
        &self,
        painter: &Painter,
        canvas_rect: Rect,
        profile: &PrinterProfile,
        doc: &crate::vector::loader::VectorDocument,
        preview_progress: Option<f32>,
        override_polylines: Option<Vec<crate::vector::loader::Polyline>>,
    ) {
        if doc.is_empty() {
            return;
        }

        let transformed = override_polylines.unwrap_or_else(|| doc.get_transformed_polylines());

        // Check bounds
        let pen_min_x = profile.toolhead_offset_x.max(0.0);
        let pen_max_x = (profile.bed_width + profile.toolhead_offset_x).min(profile.bed_width);
        let pen_min_y = profile.toolhead_offset_y.max(0.0);
        let pen_max_y = (profile.bed_height + profile.toolhead_offset_y).min(profile.bed_height);

        let mut has_out_of_bounds = false;

        // Clean white primary (#f4f4f5) / error red (#ef4444)
        let line_color = Color32::from_rgb(244, 244, 245);
        let warn_color = Color32::from_rgb(239, 68, 68);
        let dim_color = Color32::from_rgb(50, 50, 54);
        let travel_color = Color32::from_rgba_unmultiplied(59, 130, 246, 120);

        let total_segs: usize = transformed.iter().map(|p| p.points.len().saturating_sub(1)).sum();
        let limit_segs = if let Some(prog) = preview_progress {
            ((total_segs as f32) * prog.clamp(0.0, 1.0)).round() as usize
        } else {
            total_segs
        };

        let mut seg_counter = 0usize;
        let mut prev_poly_last_pt: Option<Pos2> = None;
        let mut pen_tip_screen: Option<Pos2> = None;

        for poly in &transformed {
            if poly.points.is_empty() {
                continue;
            }

            // Travel move between polylines
            if let Some(prev_end) = prev_poly_last_pt {
                if preview_progress.is_some() && seg_counter <= limit_segs {
                    let travel_start = self.bed_to_screen(prev_end, canvas_rect, profile);
                    let travel_end = self.bed_to_screen(poly.points[0], canvas_rect, profile);
                    painter.line_segment([travel_start, travel_end], Stroke::new(1.0_f32, travel_color));
                }
            }

            for w in poly.points.windows(2) {
                let p0_screen = self.bed_to_screen(w[0], canvas_rect, profile);
                let p1_screen = self.bed_to_screen(w[1], canvas_rect, profile);

                let is_safe = w[0].x >= pen_min_x && w[0].x <= pen_max_x
                    && w[0].y >= pen_min_y && w[0].y <= pen_max_y
                    && w[1].x >= pen_min_x && w[1].x <= pen_max_x
                    && w[1].y >= pen_min_y && w[1].y <= pen_max_y;

                if !is_safe {
                    has_out_of_bounds = true;
                }

                let stroke_width = (profile.pen_tip_diameter_mm.max(0.1) * self.zoom).clamp(1.0, 32.0);

                if seg_counter < limit_segs {
                    let color = if is_safe { line_color } else { warn_color };
                    painter.line_segment([p0_screen, p1_screen], Stroke::new(stroke_width, color));
                    pen_tip_screen = Some(p1_screen);
                } else {
                    painter.line_segment([p0_screen, p1_screen], Stroke::new((stroke_width * 0.7).max(1.0), dim_color));
                }

                seg_counter += 1;
            }

            prev_poly_last_pt = poly.points.last().copied();
        }

        // Pen tip indicator during preview
        if preview_progress.is_some() {
            if let Some(pen_pos) = pen_tip_screen {
                painter.circle_filled(pen_pos, 4.0, Color32::from_rgb(59, 130, 246));
                painter.circle_stroke(pen_pos, 4.0, Stroke::new(1.0_f32, Color32::WHITE));
            }
        }

        // Bounding box overlay (only if not simulating preview)
        if preview_progress.is_none() {
            if let Some(bb) = doc.bounding_box() {
                let bb_tl = self.bed_to_screen(pos2(bb.min.x, bb.max.y), canvas_rect, profile);
                let bb_br = self.bed_to_screen(pos2(bb.max.x, bb.min.y), canvas_rect, profile);
                let bb_screen = Rect::from_two_pos(bb_tl, bb_br);

                let stroke_col = if has_out_of_bounds {
                    Color32::from_rgb(239, 68, 68) // #ef4444
                } else if doc.is_dragging {
                    Color32::from_rgb(59, 130, 246) // #3b82f6
                } else {
                    Color32::from_rgb(63, 63, 70)  // #3f3f46
                };

                painter.rect_stroke(bb_screen, 0.0, Stroke::new(1.0_f32, stroke_col));

                let handle_sz = 7.0;
                let handle_fill = if doc.is_resizing {
                    Color32::from_rgb(59, 130, 246) // #3b82f6 when active
                } else {
                    Color32::from_rgb(244, 244, 245)
                };

                let corners = [
                    bb_screen.left_top(),
                    bb_screen.right_top(),
                    bb_screen.left_bottom(),
                    bb_screen.right_bottom(),
                ];
                for c in corners {
                    let h_rect = Rect::from_center_size(c, vec2(handle_sz, handle_sz));
                    painter.rect_filled(h_rect, 1.0, handle_fill);
                    painter.rect_stroke(h_rect, 1.0, Stroke::new(1.0_f32, Color32::from_rgb(24, 24, 27)));
                }

                // Dimension label at bottom of bounding box
                let dim_text = format!("{:.1} × {:.1} mm", bb.width(), bb.height());
                painter.text(
                    bb_screen.center_bottom() + vec2(0.0, 6.0),
                    egui::Align2::CENTER_TOP,
                    dim_text,
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(161, 161, 170), // #a1a1aa
                );

                // Center anchor crosshair
                let center_screen = self.bed_to_screen(doc.position, canvas_rect, profile);
                painter.line_segment(
                    [center_screen + vec2(-5.0, 0.0), center_screen + vec2(5.0, 0.0)],
                    Stroke::new(1.0_f32, stroke_col),
                );
                painter.line_segment(
                    [center_screen + vec2(0.0, -5.0), center_screen + vec2(0.0, 5.0)],
                    Stroke::new(1.0_f32, stroke_col),
                );
            }
        }
    }

    pub fn get_corner_handles(
        &self,
        canvas_rect: Rect,
        profile: &PrinterProfile,
        doc: &crate::vector::loader::VectorDocument,
    ) -> Option<[Pos2; 4]> {
        if doc.is_empty() {
            return None;
        }
        let bb = doc.bounding_box()?;
        let bb_tl = self.bed_to_screen(pos2(bb.min.x, bb.max.y), canvas_rect, profile);
        let bb_br = self.bed_to_screen(pos2(bb.max.x, bb.min.y), canvas_rect, profile);
        let bb_screen = Rect::from_two_pos(bb_tl, bb_br);
        Some([
            bb_screen.left_top(),
            bb_screen.right_top(),
            bb_screen.left_bottom(),
            bb_screen.right_bottom(),
        ])
    }

    pub fn render_ghost_preview(
        &self,
        painter: &Painter,
        canvas_rect: Rect,
        profile: &PrinterProfile,
        bed_pos: Pos2,
        half_w: f32,
        half_h: f32,
        is_circle: bool,
    ) {
        let center_screen = self.bed_to_screen(bed_pos, canvas_rect, profile);
        let color_stroke = Color32::from_rgba_unmultiplied(59, 130, 246, 180); // #3b82f6 with alpha
        let color_fill = Color32::from_rgba_unmultiplied(59, 130, 246, 30);

        if is_circle {
            let r_screen = half_w * self.zoom;
            painter.circle(center_screen, r_screen, color_fill, Stroke::new(1.0_f32, color_stroke));
        } else {
            let p_tl = self.bed_to_screen(pos2(bed_pos.x - half_w, bed_pos.y + half_h), canvas_rect, profile);
            let p_br = self.bed_to_screen(pos2(bed_pos.x + half_w, bed_pos.y - half_h), canvas_rect, profile);
            let ghost_rect = Rect::from_two_pos(p_tl, p_br);
            painter.rect(ghost_rect, 2.0, color_fill, Stroke::new(1.0_f32, color_stroke));
        }

        // Center crosshair
        painter.line_segment([center_screen + vec2(-6.0, 0.0), center_screen + vec2(6.0, 0.0)], Stroke::new(1.0_f32, color_stroke));
        painter.line_segment([center_screen + vec2(0.0, -6.0), center_screen + vec2(0.0, 6.0)], Stroke::new(1.0_f32, color_stroke));
    }

    pub fn render_bed_and_grid(
        &self,
        painter: &Painter,
        canvas_rect: Rect,
        profile: &PrinterProfile,
    ) {
        // Bed rectangle in screen coords
        let p_bl = self.bed_to_screen(pos2(0.0, 0.0), canvas_rect, profile);
        let p_br = self.bed_to_screen(pos2(profile.bed_width, 0.0), canvas_rect, profile);
        let p_tl = self.bed_to_screen(pos2(0.0, profile.bed_height), canvas_rect, profile);

        let bed_screen_rect = Rect::from_two_pos(p_tl, p_br);

        // Bed background: dark surface #141416
        painter.rect_filled(
            bed_screen_rect,
            0.0,
            Color32::from_rgb(20, 20, 23),
        );

        // Bed outline: 1px solid #3b82f6 as requested
        painter.rect_stroke(
            bed_screen_rect,
            0.0,
            Stroke::new(1.0_f32, Color32::from_rgb(59, 130, 246)),
        );

        // Grid lines with LOD: 10mm lines fade/hide when zoomed out
        let draw_10mm = self.zoom >= 0.8;
        let stroke_10mm = Stroke::new(
            0.5_f32,
            if self.zoom >= 1.5 {
                Color32::from_rgb(39, 39, 42) // #27272a
            } else {
                Color32::from_rgb(30, 30, 33)
            },
        );
        let stroke_50mm = Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)); // #3f3f46

        // Vertical lines (X)
        let mut x = 10.0;
        while x < profile.bed_width {
            let is_50 = (x % 50.0).abs() < 0.1;
            if is_50 {
                let bot = self.bed_to_screen(pos2(x, 0.0), canvas_rect, profile);
                let top = self.bed_to_screen(pos2(x, profile.bed_height), canvas_rect, profile);
                painter.line_segment([bot, top], stroke_50mm);
            } else if draw_10mm {
                let bot = self.bed_to_screen(pos2(x, 0.0), canvas_rect, profile);
                let top = self.bed_to_screen(pos2(x, profile.bed_height), canvas_rect, profile);
                painter.line_segment([bot, top], stroke_10mm);
            }
            x += 10.0;
        }

        // Horizontal lines (Y)
        let mut y = 10.0;
        while y < profile.bed_height {
            let is_50 = (y % 50.0).abs() < 0.1;
            if is_50 {
                let left = self.bed_to_screen(pos2(0.0, y), canvas_rect, profile);
                let right = self.bed_to_screen(pos2(profile.bed_width, y), canvas_rect, profile);
                painter.line_segment([left, right], stroke_50mm);
            } else if draw_10mm {
                let left = self.bed_to_screen(pos2(0.0, y), canvas_rect, profile);
                let right = self.bed_to_screen(pos2(profile.bed_width, y), canvas_rect, profile);
                painter.line_segment([left, right], stroke_10mm);
            }
            y += 10.0;
        }

        // Toolhead Reachable Area (subtle outline)
        let pen_min_x = profile.toolhead_offset_x.max(0.0);
        let pen_max_x = (profile.bed_width + profile.toolhead_offset_x).min(profile.bed_width);
        let pen_min_y = profile.toolhead_offset_y.max(0.0);
        let pen_max_y = (profile.bed_height + profile.toolhead_offset_y).min(profile.bed_height);

        if pen_min_x < pen_max_x && pen_min_y < pen_max_y {
            let reach_tl = self.bed_to_screen(pos2(pen_min_x, pen_max_y), canvas_rect, profile);
            let reach_br = self.bed_to_screen(pos2(pen_max_x, pen_min_y), canvas_rect, profile);
            let reach_rect = Rect::from_two_pos(reach_tl, reach_br);

            painter.rect_stroke(
                reach_rect,
                0.0,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(16, 185, 129, 90)), // subtle #10b981
            );
        }

        // Axes at origin (0, 0)
        let origin = p_bl;
        let x_arrow_tip = origin + vec2(30.0, 0.0);
        let y_arrow_tip = origin + vec2(0.0, -30.0);

        painter.line_segment([origin, x_arrow_tip], Stroke::new(1.5_f32, Color32::from_rgb(239, 68, 68)));
        painter.line_segment([origin, y_arrow_tip], Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));

        painter.text(
            x_arrow_tip + vec2(6.0, -2.0),
            egui::Align2::LEFT_CENTER,
            "X",
            egui::FontId::monospace(10.0),
            Color32::from_rgb(161, 161, 170),
        );
        painter.text(
            y_arrow_tip + vec2(0.0, -6.0),
            egui::Align2::CENTER_BOTTOM,
            "Y",
            egui::FontId::monospace(10.0),
            Color32::from_rgb(161, 161, 170),
        );
    }
}
