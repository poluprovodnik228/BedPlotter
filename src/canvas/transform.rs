use egui::{Color32, ColorImage, Context, Pos2, Rect, Stroke, TextureHandle, Vec2};
use image::{GenericImageView, ImageReader, Rgba, RgbaImage};
use nalgebra::{Matrix3, Vector3};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationTarget {
    FullBed,
    A4Sheet, // 210 x 297 mm
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Calibration, // 4-point perspective setup
    BedView,     // Working bed view with rectified photo & grid
}

pub struct PerspectiveCalibration {
    /// 4 points in normalized image coordinates [0.0 .. 1.0]:
    /// 0: Bottom-Left, 1: Bottom-Right, 2: Top-Right, 3: Top-Left
    pub points_norm: [Pos2; 4],
    pub active_point_idx: Option<usize>,
    pub target: CalibrationTarget,

    // Loaded raw image
    pub raw_image: Option<RgbaImage>,
    pub image_width: u32,
    pub image_height: u32,

    // Textures for egui
    pub original_texture: Option<TextureHandle>,
    pub rectified_texture: Option<TextureHandle>,

    // Calibration state
    pub homography_matrix: Option<Matrix3<f64>>,
    pub is_rectified: bool,
    pub photo_opacity: f32, // 0.0 to 1.0
}

impl Default for PerspectiveCalibration {
    fn default() -> Self {
        Self {
            points_norm: [
                Pos2::new(0.1, 0.9), // BL
                Pos2::new(0.9, 0.9), // BR
                Pos2::new(0.9, 0.1), // TR
                Pos2::new(0.1, 0.1), // TL
            ],
            active_point_idx: None,
            target: CalibrationTarget::FullBed,
            raw_image: None,
            image_width: 0,
            image_height: 0,
            original_texture: None,
            rectified_texture: None,
            homography_matrix: None,
            is_rectified: false,
            photo_opacity: 0.85,
        }
    }
}

impl PerspectiveCalibration {
    pub fn load_image_from_path(&mut self, path: &Path, ctx: &Context) -> Result<(), Box<dyn std::error::Error>> {
        let reader = ImageReader::open(path)?.with_guessed_format()?;
        let dyn_img = reader.decode()?;
        
        // Resize down if photo is gigantic (> 2400px on long edge) for UI smoothness
        let (w, h) = dyn_img.dimensions();
        let max_dim = 2400;
        let dyn_img = if w > max_dim || h > max_dim {
            dyn_img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
        } else {
            dyn_img
        };

        let rgba = dyn_img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Build egui ColorImage
        let pixels = rgba.as_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            pixels,
        );

        let texture = ctx.load_texture("bed_photo_original", color_image, egui::TextureOptions::LINEAR);

        self.raw_image = Some(rgba);
        self.image_width = width;
        self.image_height = height;
        self.original_texture = Some(texture);
        self.rectified_texture = None;
        self.is_rectified = false;

        // Reset default corners near edges
        self.points_norm = [
            Pos2::new(0.15, 0.85), // BL
            Pos2::new(0.85, 0.85), // BR
            Pos2::new(0.85, 0.15), // TR
            Pos2::new(0.15, 0.15), // TL
        ];

        Ok(())
    }

    /// Reset control points to image corners
    pub fn reset_corners(&mut self) {
        self.points_norm = [
            Pos2::new(0.05, 0.95), // BL
            Pos2::new(0.95, 0.95), // BR
            Pos2::new(0.95, 0.05), // TR
            Pos2::new(0.05, 0.05), // TL
        ];
    }

    /// Compute 3x3 homography matrix mapping normalized image coords to bed mm coords
    pub fn compute_homography(&mut self, bed_width: f64, bed_height: f64) -> Option<Matrix3<f64>> {
        let (dst_w, dst_h) = match self.target {
            CalibrationTarget::FullBed => (bed_width, bed_height),
            CalibrationTarget::A4Sheet => (210.0, 297.0),
        };

        let dst = [
            (0.0, 0.0),       // BL
            (dst_w, 0.0),     // BR
            (dst_w, dst_h),   // TR
            (0.0, dst_h),     // TL
        ];

        let src = [
            (self.points_norm[0].x as f64, self.points_norm[0].y as f64),
            (self.points_norm[1].x as f64, self.points_norm[1].y as f64),
            (self.points_norm[2].x as f64, self.points_norm[2].y as f64),
            (self.points_norm[3].x as f64, self.points_norm[3].y as f64),
        ];

        let mut a = nalgebra::DMatrix::<f64>::zeros(8, 8);
        let mut b = nalgebra::DVector::<f64>::zeros(8);

        for i in 0..4 {
            let (sx, sy) = src[i];
            let (dx, dy) = dst[i];

            a[(2 * i, 0)] = sx;
            a[(2 * i, 1)] = sy;
            a[(2 * i, 2)] = 1.0;
            a[(2 * i, 6)] = -dx * sx;
            a[(2 * i, 7)] = -dx * sy;
            b[2 * i] = dx;

            a[(2 * i + 1, 3)] = sx;
            a[(2 * i + 1, 4)] = sy;
            a[(2 * i + 1, 5)] = 1.0;
            a[(2 * i + 1, 6)] = -dy * sx;
            a[(2 * i + 1, 7)] = -dy * sy;
            b[2 * i + 1] = dy;
        }

        let decomp = a.lu();
        if let Some(h_vec) = decomp.solve(&b) {
            let h = Matrix3::new(
                h_vec[0], h_vec[1], h_vec[2],
                h_vec[3], h_vec[4], h_vec[5],
                h_vec[6], h_vec[7], 1.0,
            );
            self.homography_matrix = Some(h);
            Some(h)
        } else {
            None
        }
    }

    /// Perform rectification (unwarping) using inverse homography and bilinear interpolation
    pub fn rectify(&mut self, ctx: &Context, bed_width: f64, bed_height: f64) -> Result<(), String> {
        if self.raw_image.is_none() {
            return Err("No photo loaded to calibrate".to_string());
        }

        let h_matrix = match self.compute_homography(bed_width, bed_height) {
            Some(h) => h,
            None => return Err("Failed to solve homography matrix (singular points)".to_string()),
        };

        // Inverse homography maps (dst_x_mm, dst_y_mm) -> (norm_src_x, norm_src_y)
        let h_inv = match h_matrix.try_inverse() {
            Some(inv) => inv,
            None => return Err("Homography matrix is not invertible".to_string()),
        };

        let raw = self.raw_image.as_ref().unwrap();

        let (target_w, target_h) = match self.target {
            CalibrationTarget::FullBed => (bed_width.max(10.0), bed_height.max(10.0)),
            CalibrationTarget::A4Sheet => (210.0, 297.0),
        };

        // Resolution for rectified bed texture (e.g. ~4-5 pixels per mm)
        let out_w = 1200u32;
        let out_h = ((1200.0 * (target_h / target_w)).round() as u32).clamp(400, 2400);

        let mut out_img = RgbaImage::new(out_w, out_h);
        let src_w = self.image_width as f64;
        let src_h = self.image_height as f64;

        for out_y in 0..out_h {
            // Note: in bed coordinates, Y=0 is bottom, but in image texture Y=0 is top.
            // So dst_y_mm goes from 0 at bottom to target_h at top:
            let dst_y_mm = ((out_h - 1 - out_y) as f64 / (out_h - 1) as f64) * target_h;

            for out_x in 0..out_w {
                let dst_x_mm = (out_x as f64 / (out_w - 1) as f64) * target_w;

                let pt = Vector3::new(dst_x_mm, dst_y_mm, 1.0);
                let src_proj = h_inv * pt;
                if src_proj.z.abs() < 1e-9 {
                    continue;
                }

                let norm_u = src_proj.x / src_proj.z;
                let norm_v = src_proj.y / src_proj.z;

                if norm_u >= 0.0 && norm_u <= 1.0 && norm_v >= 0.0 && norm_v <= 1.0 {
                    // Bilinear interpolation
                    let px = norm_u * (src_w - 1.0);
                    let py = norm_v * (src_h - 1.0);

                    let x0 = px.floor() as u32;
                    let x1 = (x0 + 1).min(self.image_width - 1);
                    let y0 = py.floor() as u32;
                    let y1 = (y0 + 1).min(self.image_height - 1);

                    let fx = (px - x0 as f64) as f32;
                    let fy = (py - y0 as f64) as f32;

                    let c00 = raw.get_pixel(x0, y0);
                    let c10 = raw.get_pixel(x1, y0);
                    let c01 = raw.get_pixel(x0, y1);
                    let c11 = raw.get_pixel(x1, y1);

                    let mut sampled = [0u8; 4];
                    for c in 0..4 {
                        let top = (c00[c] as f32) * (1.0 - fx) + (c10[c] as f32) * fx;
                        let bot = (c01[c] as f32) * (1.0 - fx) + (c11[c] as f32) * fx;
                        sampled[c] = (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8;
                    }

                    out_img.put_pixel(out_x, out_y, Rgba(sampled));
                } else {
                    out_img.put_pixel(out_x, out_y, Rgba([20, 24, 30, 255]));
                }
            }
        }

        // Upload to egui GPU texture
        let color_img = ColorImage::from_rgba_unmultiplied(
            [out_w as usize, out_h as usize],
            out_img.as_raw(),
        );

        let tex = ctx.load_texture("bed_photo_rectified", color_img, egui::TextureOptions::LINEAR);
        self.rectified_texture = Some(tex);
        self.is_rectified = true;

        Ok(())
    }

    /// Render calibration view (image with draggable handles)
    pub fn render_calibration_editor(
        &mut self,
        ui: &mut egui::Ui,
        canvas_rect: Rect,
    ) {
        let painter = ui.painter_at(canvas_rect);

        // Fill background: #121214
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(18, 18, 20));

        let tex = match &self.original_texture {
            Some(t) => t,
            None => {
                painter.text(
                    canvas_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Фотография стола не загружена.\nНажмите 'Загрузить фото' на панели справа.",
                    egui::FontId::proportional(15.0),
                    Color32::from_rgb(161, 161, 170),
                );
                return;
            }
        };

        // Compute aspect-fit rect for the photo inside canvas
        let img_w = self.image_width as f32;
        let img_h = self.image_height as f32;
        let scale = ((canvas_rect.width() - 40.0) / img_w)
            .min((canvas_rect.height() - 40.0) / img_h)
            .max(0.01);

        let draw_size = Vec2::new(img_w * scale, img_h * scale);
        let img_rect = Rect::from_center_size(canvas_rect.center(), draw_size);

        // Draw photo
        painter.image(
            tex.id(),
            img_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // Map normalized points to screen
        let to_screen = |norm: Pos2| -> Pos2 {
            Pos2::new(
                img_rect.min.x + norm.x * img_rect.width(),
                img_rect.min.y + norm.y * img_rect.height(),
            )
        };

        let screen_pts: [Pos2; 4] = [
            to_screen(self.points_norm[0]),
            to_screen(self.points_norm[1]),
            to_screen(self.points_norm[2]),
            to_screen(self.points_norm[3]),
        ];

        // Draw polygon outline connecting points: 1.5px #3b82f6
        let poly_stroke = egui::Stroke::new(1.5_f32, Color32::from_rgb(59, 130, 246));
        for i in 0..4 {
            let next = (i + 1) % 4;
            painter.line_segment([screen_pts[i], screen_pts[next]], poly_stroke);
        }

        // Draggable point handles
        let handle_radius = 12.0;
        let labels = [
            ("1: BL (0, 0)", Color32::from_rgb(239, 68, 68)),
            ("2: BR (X, 0)", Color32::from_rgb(245, 158, 11)),
            ("3: TR (X, Y)", Color32::from_rgb(16, 185, 129)),
            ("4: TL (0, Y)", Color32::from_rgb(59, 130, 246)),
        ];

        let mouse_pos = ui.input(|i| i.pointer.interact_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        // Handle dragging
        if pointer_down {
            if let Some(active) = self.active_point_idx {
                if let Some(m_pos) = mouse_pos {
                    let norm_x = ((m_pos.x - img_rect.min.x) / img_rect.width()).clamp(0.0, 1.0);
                    let norm_y = ((m_pos.y - img_rect.min.y) / img_rect.height()).clamp(0.0, 1.0);
                    self.points_norm[active] = Pos2::new(norm_x, norm_y);
                }
            } else if let Some(m_pos) = mouse_pos {
                // Find nearest handle under cursor
                for (idx, &pt) in screen_pts.iter().enumerate() {
                    if pt.distance(m_pos) <= handle_radius * 1.5 {
                        self.active_point_idx = Some(idx);
                        break;
                    }
                }
            }
        } else {
            self.active_point_idx = None;
        }

        // Draw handles and text
        for i in 0..4 {
            let pt = screen_pts[i];
            let (label, col) = labels[i];
            let is_hovered = mouse_pos.map_or(false, |m| m.distance(pt) <= handle_radius * 1.5);
            let is_active = self.active_point_idx == Some(i);

            let r = if is_active || is_hovered { handle_radius + 3.0 } else { handle_radius };

            // Clean 1px border with dark fill
            painter.circle_stroke(pt, r, egui::Stroke::new(1.5_f32, col));
            painter.circle_filled(pt, r - 2.0, Color32::from_rgb(24, 24, 27));
            painter.circle_filled(pt, 3.0, col);

            // Label badge with #27272a background, border #3f3f46, text #f4f4f5
            let label_pos = pt + Vec2::new(16.0, -10.0);
            painter.rect_filled(
                Rect::from_min_size(label_pos + Vec2::new(-4.0, -2.0), Vec2::new(80.0, 18.0)),
                4.0,
                Color32::from_rgb(39, 39, 42),
            );
            painter.rect_stroke(
                Rect::from_min_size(label_pos + Vec2::new(-4.0, -2.0), Vec2::new(80.0, 18.0)),
                4.0,
                Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)),
            );
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::monospace(10.0),
                Color32::from_rgb(244, 244, 245),
            );
        }
    }
}
