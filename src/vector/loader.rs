use super::font::{FontStyle, StrokeFont};
use super::hatch::HatchGenerator;
use super::humanizer::Humanizer;
use egui::{pos2, Pos2, Rect};
use std::f32::consts::PI;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Polyline {
    pub points: Vec<Pos2>,
}

impl Polyline {
    pub fn length(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut len = 0.0;
        for w in self.points.windows(2) {
            len += w[0].distance(w[1]);
        }
        len
    }
}

/// Metadata for text items to enable real-time WYSIWYG editing
#[derive(Debug, Clone, PartialEq)]
pub struct TextItemData {
    pub text: String,
    pub font_size_mm: f32,
    pub style: FontStyle,
    pub humanizer_strength: f32,
}

/// An individual, independent vector shape/layer on the plotter bed
#[derive(Debug, Clone)]
pub struct VectorItem {
    #[allow(dead_code)]
    pub id: u64,
    pub name: String,
    pub polylines: Vec<Polyline>, // local points centered at (0, 0)
    pub position: Pos2,           // bed coordinate center in mm
    pub scale: f32,
    pub rotation_deg: f32,
    pub lock_aspect_ratio: bool,
    pub text_data: Option<TextItemData>,
}

impl VectorItem {
    pub fn get_transformed_polylines(&self) -> Vec<Polyline> {
        let rad = self.rotation_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        self.polylines
            .iter()
            .map(|poly| {
                let pts: Vec<Pos2> = poly
                    .points
                    .iter()
                    .map(|pt| {
                        let sx = pt.x * self.scale;
                        let sy = pt.y * self.scale;
                        let rx = sx * cos_a - sy * sin_a;
                        let ry = sx * sin_a + sy * cos_a;
                        pos2(self.position.x + rx, self.position.y + ry)
                    })
                    .collect();
                Polyline { points: pts }
            })
            .collect()
    }

    pub fn bounding_box(&self) -> Option<Rect> {
        let transformed = self.get_transformed_polylines();
        if transformed.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for p in &transformed {
            for pt in &p.points {
                min_x = min_x.min(pt.x);
                max_x = max_x.max(pt.x);
                min_y = min_y.min(pt.y);
                max_y = max_y.max(pt.y);
            }
        }

        if min_x > max_x {
            None
        } else {
            Some(Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y)))
        }
    }

    #[allow(dead_code)]
    pub fn get_dimensions_mm(&self) -> (f32, f32) {
        if let Some(bb) = self.bounding_box() {
            (bb.width(), bb.height())
        } else {
            (0.0, 0.0)
        }
    }

    pub fn total_length_mm(&self) -> f32 {
        self.polylines.iter().map(|p| p.length()).sum::<f32>() * self.scale
    }

    /// Re-render vector polylines in real time when text or typography settings change
    pub fn update_text(
        &mut self,
        text: &str,
        font_size_mm: f32,
        style: FontStyle,
        humanizer_strength: f32,
    ) {
        let mut polylines = StrokeFont::render_text_styled(text, font_size_mm, style);
        if humanizer_strength > 0.001 && !polylines.is_empty() {
            polylines = Humanizer::humanize_polylines(&polylines, humanizer_strength);
        }
        VectorDocument::normalize_and_center_cartesian(&mut polylines);
        self.polylines = polylines;

        let clean = text.replace('\n', " ");
        let preview = if clean.chars().count() > 18 {
            format!("{}...", clean.chars().take(18).collect::<String>())
        } else if clean.is_empty() {
            "(пустой)".to_string()
        } else {
            clean
        };
        self.name = format!("Текст: \"{}\"", preview);
        self.text_data = Some(TextItemData {
            text: text.to_string(),
            font_size_mm,
            style,
            humanizer_strength,
        });
    }
}

#[derive(Debug, Clone)]
pub struct VectorDocument {
    /// Independent objects / layers on the bed
    pub items: Vec<VectorItem>,
    /// Currently active / selected item index
    pub selected_index: Option<usize>,
    /// Auto-increment ID generator
    pub next_id: u64,
    /// Document change revision counter for zero-cost caching
    pub version: u64,

    // Active selected item proxy fields (for backward compatibility)
    pub name: String,
    pub raw_polylines: Vec<Polyline>,
    pub position: Pos2,
    pub scale: f32,
    pub rotation_deg: f32,
    pub is_dragging: bool,
    pub is_resizing: bool,
    pub lock_aspect_ratio: bool,
}

impl Default for VectorDocument {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            next_id: 1,
            version: 1,
            name: "Empty".to_string(),
            raw_polylines: Vec::new(),
            position: Pos2::new(111.0, 111.0),
            scale: 1.0,
            rotation_deg: 0.0,
            is_dragging: false,
            is_resizing: false,
            lock_aspect_ratio: true,
        }
    }
}

impl VectorDocument {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = None;
        self.name = "Empty".to_string();
        self.raw_polylines.clear();
        self.scale = 1.0;
        self.rotation_deg = 0.0;
        self.is_dragging = false;
        self.is_resizing = false;
        self.version = self.version.wrapping_add(1);
    }

    pub fn selected_item(&self) -> Option<&VectorItem> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }

    #[allow(dead_code)]
    pub fn selected_item_mut(&mut self) -> Option<&mut VectorItem> {
        let idx = self.selected_index?;
        self.items.get_mut(idx)
    }

    /// Synchronize the proxy fields from the currently selected item
    pub fn sync_from_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(item) = self.items.get(idx) {
                self.name = item.name.clone();
                self.position = item.position;
                self.scale = item.scale;
                self.rotation_deg = item.rotation_deg;
                self.lock_aspect_ratio = item.lock_aspect_ratio;
                self.raw_polylines = item.polylines.clone();
                return;
            }
        }
        self.name = "Empty".to_string();
        self.raw_polylines.clear();
    }

    /// Synchronize modifications from proxy fields back into the active selected item
    pub fn sync_to_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(item) = self.items.get_mut(idx) {
                item.position = self.position;
                item.scale = self.scale;
                item.rotation_deg = self.rotation_deg;
                item.lock_aspect_ratio = self.lock_aspect_ratio;
                self.version = self.version.wrapping_add(1);
            }
        }
    }

    /// Add a new independent item to the scene
    pub fn add_item(&mut self, name: &str, polylines: Vec<Polyline>, position: Pos2) {
        if polylines.is_empty() {
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        let item = VectorItem {
            id,
            name: name.to_string(),
            polylines,
            position,
            scale: 1.0,
            rotation_deg: 0.0,
            lock_aspect_ratio: true,
            text_data: None,
        };
        self.items.push(item);
        self.selected_index = Some(self.items.len() - 1);
        self.version = self.version.wrapping_add(1);
        self.sync_from_selected();
    }

    /// Update text, font, size, and humanizer of the currently selected item in real time
    pub fn update_selected_text(
        &mut self,
        text: &str,
        font_size_mm: f32,
        style: FontStyle,
        humanizer_strength: f32,
    ) {
        if let Some(idx) = self.selected_index {
            if let Some(item) = self.items.get_mut(idx) {
                item.update_text(text, font_size_mm, style, humanizer_strength);
                self.version = self.version.wrapping_add(1);
            }
        }
        self.sync_from_selected();
    }

    /// Access text metadata of currently selected item (if it is a text object)
    pub fn selected_text_data(&self) -> Option<&TextItemData> {
        self.selected_item().and_then(|it| it.text_data.as_ref())
    }

    /// Select an item by index
    pub fn select_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
            self.sync_from_selected();
        }
    }

    #[allow(dead_code)]
    pub fn deselect(&mut self) {
        self.selected_index = None;
        self.sync_from_selected();
    }

    /// Delete a single item by index
    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.version = self.version.wrapping_add(1);
            if self.items.is_empty() {
                self.selected_index = None;
                self.clear();
            } else {
                let new_idx = index.min(self.items.len() - 1);
                self.selected_index = Some(new_idx);
                self.sync_from_selected();
            }
        }
    }

    /// Combine / Glue all independent objects into a single unified object (Merge)
    pub fn merge_all(&mut self) {
        if self.items.len() <= 1 {
            return;
        }
        let all_polylines = self.get_transformed_polylines();
        let bb = self.bounding_box_all();
        let center = if let Some(b) = bb { b.center() } else { pos2(100.0, 100.0) };

        // Normalize points relative to center
        let local_polylines: Vec<Polyline> = all_polylines
            .into_iter()
            .map(|p| {
                let pts = p.points.into_iter().map(|pt| pos2(pt.x - center.x, pt.y - center.y)).collect();
                Polyline { points: pts }
            })
            .collect();

        self.items.clear();
        let id = self.next_id;
        self.next_id += 1;
        let merged = VectorItem {
            id,
            name: "Сгруппированная модель (Merge)".to_string(),
            polylines: local_polylines,
            position: center,
            scale: 1.0,
            rotation_deg: 0.0,
            lock_aspect_ratio: true,
            text_data: None,
        };
        self.items.push(merged);
        self.selected_index = Some(0);
        self.version = self.version.wrapping_add(1);
        self.sync_from_selected();
    }

    /// Union bounding box of all items on the bed
    pub fn bounding_box_all(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut has_any = false;

        for item in &self.items {
            if let Some(bb) = item.bounding_box() {
                has_any = true;
                min_x = min_x.min(bb.min.x);
                max_x = max_x.max(bb.max.x);
                min_y = min_y.min(bb.min.y);
                max_y = max_y.max(bb.max.y);
            }
        }

        if has_any {
            Some(Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y)))
        } else {
            None
        }
    }

    /// Hit-test: find top-most item containing bed_pos
    pub fn item_at_bed_pos(&self, bed_pos: Pos2) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate().rev() {
            if let Some(bb) = item.bounding_box() {
                if bb.expand(5.0).contains(bed_pos) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Load and tessellate SVG file into polylines as an independent item
    pub fn load_svg(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let svg_data = fs::read(path)?;
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(&svg_data, &opt)?;

        let mut extracted = Vec::new();
        Self::traverse_group(tree.root(), &mut extracted);

        if extracted.is_empty() {
            return Err("No drawable vector paths found in SVG".into());
        }

        // Center polylines around (0, 0) and flip SVG's top-down Y coordinate
        Self::normalize_and_center_svg(&mut extracted);

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let pos = self.position;
        self.add_item(&file_name, extracted, pos);

        Ok(())
    }

    fn traverse_group(group: &usvg::Group, extracted: &mut Vec<Polyline>) {
        for node in group.children() {
            match node {
                usvg::Node::Group(ref g) => Self::traverse_group(g, extracted),
                usvg::Node::Path(ref path) => Self::extract_path(path, extracted),
                _ => {}
            }
        }
    }

    fn extract_path(path: &usvg::Path, extracted: &mut Vec<Polyline>) {
        let mut current_subpath = Vec::new();
        let mut current_pt = Pos2::ZERO;

        for seg in path.data().segments() {
            match seg {
                usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                    if current_subpath.len() >= 2 {
                        extracted.push(Polyline { points: current_subpath });
                        current_subpath = Vec::new();
                    } else {
                        current_subpath.clear();
                    }
                    let pt = pos2(p.x, p.y);
                    current_pt = pt;
                    current_subpath.push(pt);
                }
                usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                    let pt = pos2(p.x, p.y);
                    current_pt = pt;
                    current_subpath.push(pt);
                }
                usvg::tiny_skia_path::PathSegment::QuadTo(p1, p2) => {
                    let p0 = current_pt;
                    let p1 = pos2(p1.x, p1.y);
                    let p2 = pos2(p2.x, p2.y);
                    let steps = 12;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let inv = 1.0 - t;
                        let x = inv * inv * p0.x + 2.0 * inv * t * p1.x + t * t * p2.x;
                        let y = inv * inv * p0.y + 2.0 * inv * t * p1.y + t * t * p2.y;
                        current_subpath.push(pos2(x, y));
                    }
                    current_pt = p2;
                }
                usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p3) => {
                    let p0 = current_pt;
                    let p1 = pos2(p1.x, p1.y);
                    let p2 = pos2(p2.x, p2.y);
                    let p3 = pos2(p3.x, p3.y);
                    let steps = 18;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let inv = 1.0 - t;
                        let x = inv * inv * inv * p0.x
                            + 3.0 * inv * inv * t * p1.x
                            + 3.0 * inv * t * t * p2.x
                            + t * t * t * p3.x;
                        let y = inv * inv * inv * p0.y
                            + 3.0 * inv * inv * t * p1.y
                            + 3.0 * inv * t * t * p2.y
                            + t * t * t * p3.y;
                        current_subpath.push(pos2(x, y));
                    }
                    current_pt = p3;
                }
                usvg::tiny_skia_path::PathSegment::Close => {
                    if let Some(&first) = current_subpath.first() {
                        current_subpath.push(first);
                    }
                    if current_subpath.len() >= 2 {
                        extracted.push(Polyline { points: current_subpath });
                        current_subpath = Vec::new();
                    }
                }
            }
        }

        if current_subpath.len() >= 2 {
            extracted.push(Polyline { points: current_subpath });
        }
    }

    /// Add a new independent shape/layer at target position on the bed
    pub fn append_shape_at(
        &mut self,
        name: &str,
        new_polylines: Vec<Polyline>,
        target_bed_pos: Pos2,
    ) {
        if new_polylines.is_empty() {
            return;
        }
        self.add_item(name, new_polylines, target_bed_pos);
    }

    /// Add basic shape: Rectangle with optional Hatch Infill at target position
    pub fn add_rectangle_at(
        &mut self,
        width: f32,
        height: f32,
        pos: Pos2,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        let hw = width * 0.5;
        let hh = height * 0.5;
        let border_pts = vec![
            pos2(-hw, -hh),
            pos2(hw, -hh),
            pos2(hw, hh),
            pos2(-hw, hh),
            pos2(-hw, -hh),
        ];

        let mut polylines = vec![Polyline {
            points: border_pts.clone(),
        }];

        if fill_hatch {
            let mut infill = HatchGenerator::generate_infill(
                &border_pts[..4],
                angle_deg,
                spacing_mm,
                cross_hatch,
            );
            polylines.append(&mut infill);
        }

        self.append_shape_at(&format!("Rectangle {:.0}x{:.0}mm", width, height), polylines, pos);
    }

    /// Add basic shape: Rectangle with optional Hatch Infill
    #[allow(dead_code)]
    pub fn add_rectangle(&mut self, width: f32, height: f32) {
        self.add_rectangle_advanced(width, height, false, 45.0, 1.5, false);
    }

    pub fn add_rectangle_advanced(
        &mut self,
        width: f32,
        height: f32,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        self.add_rectangle_at(width, height, self.position, fill_hatch, angle_deg, spacing_mm, cross_hatch);
    }

    /// Add basic shape: Circle with optional Hatch Infill at target position
    pub fn add_circle_at(
        &mut self,
        radius: f32,
        pos: Pos2,
        segments: usize,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        let segs = segments.max(16);
        let mut pts = Vec::with_capacity(segs + 1);
        for i in 0..=segs {
            let angle = (i as f32 / segs as f32) * PI * 2.0;
            pts.push(pos2(angle.cos() * radius, angle.sin() * radius));
        }

        let mut polylines = vec![Polyline { points: pts.clone() }];

        if fill_hatch {
            let mut infill = HatchGenerator::generate_infill(
                &pts[..segs],
                angle_deg,
                spacing_mm,
                cross_hatch,
            );
            polylines.append(&mut infill);
        }

        self.append_shape_at(&format!("Circle R{:.0}mm", radius), polylines, pos);
    }

    /// Add basic shape: Circle with optional Hatch Infill
    #[allow(dead_code)]
    pub fn add_circle(&mut self, radius: f32, segments: usize) {
        self.add_circle_advanced(radius, segments, false, 45.0, 1.5, false);
    }

    pub fn add_circle_advanced(
        &mut self,
        radius: f32,
        segments: usize,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        self.add_circle_at(radius, self.position, segments, fill_hatch, angle_deg, spacing_mm, cross_hatch);
    }

    /// Add basic shape: Star with optional Hatch Infill at target position
    pub fn add_star_at(
        &mut self,
        outer_r: f32,
        inner_r: f32,
        num_points: usize,
        pos: Pos2,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        let points_count = num_points.max(3);
        let total = points_count * 2;
        let mut pts = Vec::with_capacity(total + 1);

        for i in 0..=total {
            let angle = (i as f32 / total as f32) * PI * 2.0 - (PI * 0.5);
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            pts.push(pos2(angle.cos() * r, angle.sin() * r));
        }

        let mut polylines = vec![Polyline { points: pts.clone() }];

        if fill_hatch {
            let mut infill = HatchGenerator::generate_infill(
                &pts[..total],
                angle_deg,
                spacing_mm,
                cross_hatch,
            );
            polylines.append(&mut infill);
        }

        self.append_shape_at(&format!("{}-Point Star", points_count), polylines, pos);
    }

    /// Add basic shape: Star with optional Hatch Infill
    #[allow(dead_code)]
    pub fn add_star(&mut self, outer_r: f32, inner_r: f32, num_points: usize) {
        self.add_star_advanced(outer_r, inner_r, num_points, false, 45.0, 1.5, false);
    }

    pub fn add_star_advanced(
        &mut self,
        outer_r: f32,
        inner_r: f32,
        num_points: usize,
        fill_hatch: bool,
        angle_deg: f32,
        spacing_mm: f32,
        cross_hatch: bool,
    ) {
        self.add_star_at(outer_r, inner_r, num_points, self.position, fill_hatch, angle_deg, spacing_mm, cross_hatch);
    }

    /// Add bed calibration / test pattern (grid of lines + diagonals) at target position
    pub fn add_test_pattern_at(&mut self, size: f32, step: f32, pos: Pos2) {
        let mut polylines = Vec::new();
        let hs = size * 0.5;

        // Outer border
        polylines.push(Polyline {
            points: vec![
                pos2(-hs, -hs),
                pos2(hs, -hs),
                pos2(hs, hs),
                pos2(-hs, hs),
                pos2(-hs, -hs),
            ],
        });

        // Grid lines
        let mut x = -hs + step;
        while x < hs - 0.1 {
            polylines.push(Polyline {
                points: vec![pos2(x, -hs), pos2(x, hs)],
            });
            x += step;
        }

        let mut y = -hs + step;
        while y < hs - 0.1 {
            polylines.push(Polyline {
                points: vec![pos2(-hs, y), pos2(hs, y)],
            });
            y += step;
        }

        // Diagonals (X)
        polylines.push(Polyline {
            points: vec![pos2(-hs, -hs), pos2(hs, hs)],
        });
        polylines.push(Polyline {
            points: vec![pos2(-hs, hs), pos2(hs, -hs)],
        });

        self.append_shape_at(&format!("Test Pattern {:.0}mm", size), polylines, pos);
    }

    /// Add bed calibration / test pattern (grid of lines + diagonals)
    pub fn add_test_pattern(&mut self, size: f32, step: f32) {
        self.add_test_pattern_at(size, step, self.position);
    }

    /// Add vector text (Technical or Cursive style) at target position as an editable live object
    pub fn add_text_at(
        &mut self,
        text: &str,
        font_size_mm: f32,
        pos: Pos2,
        style: FontStyle,
        humanizer_strength: f32,
    ) {
        let mut polylines = StrokeFont::render_text_styled(text, font_size_mm, style);
        if humanizer_strength > 0.001 && !polylines.is_empty() {
            polylines = Humanizer::humanize_polylines(&polylines, humanizer_strength);
        }
        Self::normalize_and_center_cartesian(&mut polylines);

        let clean = text.replace('\n', " ");
        let preview = if clean.chars().count() > 18 {
            format!("{}...", clean.chars().take(18).collect::<String>())
        } else if clean.is_empty() {
            "(пустой)".to_string()
        } else {
            clean
        };

        let id = self.next_id;
        self.next_id += 1;
        let item = VectorItem {
            id,
            name: format!("Текст: \"{}\"", preview),
            polylines,
            position: pos,
            scale: 1.0,
            rotation_deg: 0.0,
            lock_aspect_ratio: true,
            text_data: Some(TextItemData {
                text: text.to_string(),
                font_size_mm,
                style,
                humanizer_strength,
            }),
        };
        self.items.push(item);
        self.selected_index = Some(self.items.len() - 1);
        self.sync_from_selected();
    }

    /// Add single-line vector text (Technical or Cursive style)
    #[allow(dead_code)]
    pub fn add_text(&mut self, text: &str, font_size_mm: f32) {
        self.add_text_styled(text, font_size_mm, FontStyle::TechnicalGost, 0.0);
    }

    pub fn add_text_styled(
        &mut self,
        text: &str,
        font_size_mm: f32,
        style: FontStyle,
        humanizer_strength: f32,
    ) {
        self.add_text_at(text, font_size_mm, self.position, style, humanizer_strength);
    }

    /// Apply organic humanizer jitter to current raw document polylines and selected item
    #[allow(dead_code)]
    pub fn apply_humanizer(&mut self, strength: f32) {
        if strength > 0.001 {
            if let Some(idx) = self.selected_index {
                if let Some(item) = self.items.get_mut(idx) {
                    item.polylines = Humanizer::humanize_polylines(&item.polylines, strength);
                    self.sync_from_selected();
                    return;
                }
            }
            if !self.raw_polylines.is_empty() {
                self.raw_polylines = Humanizer::humanize_polylines(&self.raw_polylines, strength);
            }
        }
    }

    /// Shift Cartesian coordinates so (0,0) is at geometric center (WITHOUT flipping Y)
    pub fn normalize_and_center_cartesian(polylines: &mut [Polyline]) {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for p in polylines.iter() {
            for pt in &p.points {
                min_x = min_x.min(pt.x);
                max_x = max_x.max(pt.x);
                min_y = min_y.min(pt.y);
                max_y = max_y.max(pt.y);
            }
        }

        if min_x > max_x {
            return;
        }

        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;

        for p in polylines.iter_mut() {
            for pt in &mut p.points {
                pt.x -= cx;
                pt.y -= cy;
            }
        }
    }

    /// Shift SVG coordinates so (0,0) is at center, and flip SVG Y-down to Cartesian Y-up
    fn normalize_and_center_svg(polylines: &mut [Polyline]) {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for p in polylines.iter() {
            for pt in &p.points {
                min_x = min_x.min(pt.x);
                max_x = max_x.max(pt.x);
                min_y = min_y.min(pt.y);
                max_y = max_y.max(pt.y);
            }
        }

        if min_x > max_x {
            return;
        }

        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;

        for p in polylines.iter_mut() {
            for pt in &mut p.points {
                pt.x -= cx;
                pt.y = -(pt.y - cy);
            }
        }
    }

    /// Get polylines transformed into Bed mm Coordinates from all items
    pub fn get_transformed_polylines(&self) -> Vec<Polyline> {
        self.items
            .iter()
            .flat_map(|it| it.get_transformed_polylines())
            .collect()
    }

    /// Calculate bounding box in Bed coordinates (selected item if any, otherwise all items)
    pub fn bounding_box(&self) -> Option<Rect> {
        if let Some(item) = self.selected_item() {
            item.bounding_box()
        } else {
            self.bounding_box_all()
        }
    }

    pub fn get_dimensions_mm(&self) -> (f32, f32) {
        if let Some(bb) = self.bounding_box() {
            (bb.width(), bb.height())
        } else {
            (0.0, 0.0)
        }
    }

    pub fn set_width_mm(&mut self, new_w: f32) {
        let (cur_w, _) = self.get_dimensions_mm();
        if cur_w > 0.01 && new_w > 0.1 {
            self.scale *= new_w / cur_w;
            self.sync_to_selected();
        }
    }

    pub fn set_height_mm(&mut self, new_h: f32) {
        let (_, cur_h) = self.get_dimensions_mm();
        if cur_h > 0.01 && new_h > 0.1 {
            self.scale *= new_h / cur_h;
            self.sync_to_selected();
        }
    }

    pub fn total_length_mm(&self) -> f32 {
        self.items.iter().map(|p| p.total_length_mm()).sum::<f32>()
    }

    #[allow(dead_code)]
    pub fn center_on_bed(&mut self, bed_w: f32, bed_h: f32) {
        self.position = pos2(bed_w * 0.5, bed_h * 0.5);
        self.sync_to_selected();
    }

    pub fn center_on_safe_area(&mut self, profile: &crate::config::PrinterProfile) {
        let (cx, cy) = profile.safe_pen_center();
        self.position = pos2(cx, cy);
        self.sync_to_selected();
    }

    pub fn fit_to_safe_area(&mut self, profile: &crate::config::PrinterProfile, margin: f32) {
        if self.items.is_empty() {
            return;
        }
        self.scale = 1.0;
        self.rotation_deg = 0.0;
        self.center_on_safe_area(profile);

        let (safe_w, safe_h) = profile.safe_pen_dimensions();
        if let Some(bb) = self.bounding_box() {
            let max_w = (safe_w - margin * 2.0).max(10.0);
            let max_h = (safe_h - margin * 2.0).max(10.0);
            let scale_x = max_w / bb.width().max(1.0);
            let scale_y = max_h / bb.height().max(1.0);
            self.scale = scale_x.min(scale_y).clamp(0.01, 10.0);
            self.sync_to_selected();
        }
    }

    #[allow(dead_code)]
    pub fn fit_to_bed(&mut self, bed_w: f32, bed_h: f32, margin: f32) {
        if self.items.is_empty() {
            return;
        }
        self.scale = 1.0;
        self.rotation_deg = 0.0;
        self.center_on_bed(bed_w, bed_h);

        if let Some(bb) = self.bounding_box() {
            let max_w = (bed_w - margin * 2.0).max(10.0);
            let max_h = (bed_h - margin * 2.0).max(10.0);
            let scale_x = max_w / bb.width().max(1.0);
            let scale_y = max_h / bb.height().max(1.0);
            self.scale = scale_x.min(scale_y);
            self.sync_to_selected();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multishape_composition_does_not_erase_previous_shapes() {
        let mut doc = VectorDocument::default();
        assert!(doc.is_empty());

        // 1. Add A5 rectangle
        doc.add_rectangle_at(148.0, 210.0, pos2(111.0, 111.0), false, 45.0, 1.5, false);
        assert_eq!(doc.items.len(), 1);

        // 2. Add text
        doc.add_text_at("BedPlotter", 15.0, pos2(111.0, 150.0), FontStyle::TechnicalGost, 0.0);
        assert_eq!(doc.items.len(), 2);

        // 3. Add circle
        doc.add_circle_at(30.0, pos2(111.0, 80.0), 32, false, 45.0, 1.5, false);
        assert_eq!(doc.items.len(), 3);

        // All 3 shapes are independent!
        assert!(doc.items[0].name.contains("Rectangle"));
        assert!(doc.items[1].name.contains("Текст") || doc.items[1].name.contains("Text"));
        assert!(doc.items[2].name.contains("Circle"));

        // Both previous shapes are still here and bounding box contains them all!
        let bb = doc.bounding_box_all().expect("Bounding box must exist");
        assert!(bb.width() >= 147.0);

        // Test manual merge on demand
        doc.merge_all();
        assert_eq!(doc.items.len(), 1);
        assert_eq!(doc.items[0].name, "Сгруппированная модель (Merge)");
    }
}
