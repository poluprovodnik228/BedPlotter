use crate::canvas::transform::{AppMode, CalibrationTarget, PerspectiveCalibration};
use crate::canvas::viewport::Viewport;
use crate::config::PrinterProfile;
use crate::gcode::preview::GcodePreview;
use crate::i18n::{Language, Tr};
use crate::vector::font::FontStyle;
use crate::vector::loader::VectorDocument;
use crate::vector::optimizer::PathOptimizer;
use egui::{
    pos2, vec2, Align, CentralPanel, Color32, Context, FontFamily, FontId, Key, Layout, Rect,
    RichText, Rounding, SidePanel, Stroke, TextStyle, TopBottomPanel, Ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTab {
    Prepare,   // Подготовка
    Calibrate, // Калибровка
    Preview,   // Предпросмотр (симуляция, слои, тайминги)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolId {
    Select,    // ↖ Выделение и трансформация
    Text,      // T Однолинейный текст
    Rectangle, // ▭ Прямоугольник
    Circle,    // ○ Круг
    Star,      // ☆ Звезда
    TestGrid,  // ⊞ Тестовая сетка
}

pub struct BedPlotterApp {
    pub config: PrinterProfile,
    pub viewport: Viewport,
    pub calibration: PerspectiveCalibration,
    pub vector_doc: VectorDocument,
    pub mode: AppMode,
    pub active_tab: StepTab,
    pub active_tool: ToolId,
    pub cursor_bed_pos: Option<egui::Pos2>,
    pub status_message: String,

    // Corner handle resize tracking
    pub active_resize_corner: Option<usize>,

    // G-code Simulation & Optimization
    pub preview: GcodePreview,
    pub optimize_paths: bool,
    pub cached_opt_paths: Option<Vec<crate::vector::loader::Polyline>>,
    pub doc_version: u64,
    pub cached_opt_version: u64,

    // Dialogs & Wizard
    pub show_setup_wizard: bool,
    pub show_about_dialog: bool,
    pub show_shortcuts_dialog: bool,

    // UI creation inputs
    pub text_input: String,
    pub text_size_mm: f32,
    pub rect_w: f32,
    pub rect_h: f32,
    pub circle_r: f32,
    pub star_r_out: f32,
    pub star_r_in: f32,
    pub star_pts: usize,
    pub test_size: f32,
    pub test_step: f32,

    // Feature extensions: Font Style & Humanizer
    pub font_style: FontStyle,
    pub humanizer_enabled: bool,
    pub humanizer_strength: f32,

    // Feature extensions: Hatch Infill
    pub hatch_enabled: bool,
    pub hatch_angle: f32,
    pub hatch_spacing: f32,
    pub hatch_cross: bool,

    // Feature extensions: Multipass
    pub multipass_count: usize,
}

impl BedPlotterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::apply_dark_desktop_theme(&cc.egui_ctx);

        let config = PrinterProfile::load();
        let lang = config.language;
        let show_setup_wizard = config.is_first_run;

        let mut vector_doc = VectorDocument::default();
        vector_doc.position = pos2(config.bed_width * 0.5, config.bed_height * 0.5);

        Self {
            config,
            viewport: Viewport::default(),
            calibration: PerspectiveCalibration::default(),
            vector_doc,
            mode: AppMode::BedView,
            active_tab: StepTab::Prepare,
            active_tool: ToolId::Select,
            cursor_bed_pos: None,
            status_message: Tr::ready_status(lang).to_string(),

            active_resize_corner: None,

            preview: GcodePreview::new(),
            optimize_paths: true,
            cached_opt_paths: None,
            doc_version: 1,
            cached_opt_version: 0,

            show_setup_wizard,
            show_about_dialog: false,
            show_shortcuts_dialog: false,

            text_input: "ANYCUBIC KOBRA".to_string(),
            text_size_mm: 14.0,
            rect_w: 60.0,
            rect_h: 40.0,
            circle_r: 25.0,
            star_r_out: 30.0,
            star_r_in: 14.0,
            star_pts: 5,
            test_size: 80.0,
            test_step: 10.0,

            font_style: FontStyle::Cursive,
            humanizer_enabled: false,
            humanizer_strength: 0.35,

            hatch_enabled: false,
            hatch_angle: 45.0,
            hatch_spacing: 1.5,
            hatch_cross: false,

            multipass_count: 1,
        }
    }

    /// Embed crisp official Inter font (Regular + SemiBold) with AppleSymbols fallback
    fn setup_custom_fonts(ctx: &Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "Inter-Regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        );
        fonts.font_data.insert(
            "Inter-SemiBold".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
        );
        fonts.font_data.insert(
            "AppleSymbols".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/AppleSymbols.ttf")),
        );

        // Put Inter-Regular as first choice for Proportional text, followed by AppleSymbols fallback
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "Inter-Regular".to_owned());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("AppleSymbols".to_owned());

        // Put Inter-SemiBold for emphasis & buttons, followed by AppleSymbols fallback
        fonts
            .families
            .entry(FontFamily::Name("Inter-SemiBold".into()))
            .or_default()
            .insert(0, "Inter-SemiBold".to_owned());
        fonts
            .families
            .entry(FontFamily::Name("Inter-SemiBold".into()))
            .or_default()
            .push("AppleSymbols".to_owned());

        ctx.set_fonts(fonts);
    }

    /// Strict Dark Desktop Theme (OrcaSlicer / Bambu Studio / Linear)
    fn apply_dark_desktop_theme(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        // Base backgrounds & surfaces
        visuals.override_text_color = Some(Color32::from_rgb(244, 244, 245)); // #f4f4f5
        visuals.panel_fill = Color32::from_rgb(24, 24, 27);                  // #18181b
        visuals.window_fill = Color32::from_rgb(24, 24, 27);                 // #18181b
        visuals.extreme_bg_color = Color32::from_rgb(39, 39, 42);            // #27272a (inputs)

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 24, 27);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)); // 1px #3f3f46
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        // Inactive widgets
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(39, 39, 42); // #27272a
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70));
        visuals.widgets.inactive.rounding = Rounding::same(6.0);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(244, 244, 245));

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 54);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70));
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        // Active/Pressed widgets (calm blue #3b82f6)
        visuals.widgets.active.bg_fill = Color32::from_rgb(59, 130, 246);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70));
        visuals.widgets.active.rounding = Rounding::same(6.0);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = Color32::from_rgb(59, 130, 246);
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = vec2(8.0, 8.0);
        style.spacing.button_padding = vec2(10.0, 6.0);

        // Connect genuine Inter-SemiBold to Button and Heading styles!
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Name("Inter-SemiBold".into())),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(15.0, FontFamily::Name("Inter-SemiBold".into())),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );

        ctx.set_style(style);
    }

    /// True OrcaSlicer / Bambu Studio Header (44px)
    /// Rigid 3-zone layout:
    /// - LEFT: Menu-bar button [Меню ▾] (strictly NO "BedPlotter" duplicate text)
    /// - CENTER: Pipeline tabs [ Подготовка | Калибровка | Предпросмотр ] strictly centered on panel_rect
    /// - RIGHT: Printer selector ComboBox [ Model (WxH) ] + CTA [ Экспорт G-code ]
    fn render_unified_header(&mut self, ui: &mut Ui, ctx: &Context) {
        let panel_rect = ui.max_rect();
        let lang = self.config.language;

        // 1. LEFT ZONE: Menu-bar [Меню ▾] ONLY (Width: 100px)
        let left_rect = Rect::from_min_max(
            panel_rect.left_top(),
            pos2(panel_rect.left() + 100.0, panel_rect.bottom()),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.menu_button(
                    RichText::new(Tr::menu(lang))
                        .font(FontId::new(12.0, FontFamily::Name("Inter-SemiBold".into())))
                        .color(Color32::from_rgb(244, 244, 245)),
                    |ui| {
                        if ui.button(Tr::import_svg(lang)).clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).pick_file() {
                                let _ = self.vector_doc.load_svg(&path);
                                self.vector_doc.center_on_safe_area(&self.config);
                                self.status_message = format!("SVG: {}", self.vector_doc.name);
                            }
                        }
                        if ui.button(Tr::load_bed_photo(lang)).clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new().add_filter("Images", &["jpg", "png", "webp"]).pick_file() {
                                let _ = self.calibration.load_image_from_path(&path, ctx);
                                self.active_tab = StepTab::Calibrate;
                                self.mode = AppMode::Calibration;
                            }
                        }
                        ui.separator();
                        if ui.button(Tr::printer_wizard(lang)).clicked() {
                            self.show_setup_wizard = true;
                            ui.close_menu();
                        }
                        if ui.button(Tr::shortcuts(lang)).clicked() {
                            self.show_shortcuts_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button(Tr::about(lang)).clicked() {
                            self.show_about_dialog = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(Tr::language(lang), |ui| {
                            if ui.selectable_label(self.config.language == Language::Russian, "Русский (RU)").clicked() {
                                self.config.language = Language::Russian;
                                let _ = self.config.save();
                                self.status_message = Tr::ready_status(Language::Russian).to_string();
                                ui.close_menu();
                            }
                            if ui.selectable_label(self.config.language == Language::English, "English (EN)").clicked() {
                                self.config.language = Language::English;
                                let _ = self.config.save();
                                self.status_message = Tr::ready_status(Language::English).to_string();
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        let exit_text = match lang {
                            Language::Russian => "Выход",
                            Language::English => "Quit",
                        };
                        if ui.button(exit_text).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    },
                );
            });
        });

        // 2. RIGHT ZONE: Fixed width 350px (Printer Selector ComboBox + Emerald CTA Export Button)
        let right_w = 350.0;
        let right_rect = Rect::from_min_max(
            pos2(panel_rect.right() - right_w, panel_rect.top()),
            panel_rect.right_bottom(),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Emerald CTA Button (OrcaSlicer #10b981)
                let is_ready = !self.vector_doc.is_empty();
                let (btn_fill, btn_fg) = if is_ready {
                    (Color32::from_rgb(16, 185, 129), Color32::from_rgb(9, 9, 11)) // #10b981 emerald
                } else {
                    (Color32::from_rgb(39, 39, 42), Color32::from_rgb(113, 113, 122)) // muted #71717a
                };

                let btn_export = egui::Button::new(
                    RichText::new(Tr::export_gcode(lang))
                        .small()
                        .color(btn_fg),
                )
                .fill(btn_fill)
                .stroke(Stroke::NONE)
                .rounding(Rounding::same(6.0));

                let resp_export = ui.add_enabled(is_ready, btn_export);
                if resp_export.clicked() {
                    self.export_gcode();
                }

                ui.add_space(10.0);

                // Printer selector dropdown ComboBox: strict canonical format "Anycubic Kobra 1 (222×222)"
                let current_printer_label = self.config.formatted_name();

                egui::ComboBox::from_id_salt("header_printer_dropdown")
                    .width(200.0)
                    .selected_text(
                        RichText::new(&current_printer_label)
                            .small()
                            .color(Color32::from_rgb(244, 244, 245)),
                    )
                    .show_ui(ui, |ui| {
                        for preset in PrinterProfile::presets() {
                            let is_active = self.config.clean_name() == preset.clean_name();
                            let preset_label = preset.formatted_name();
                            if ui.selectable_label(is_active, preset_label).clicked() {
                                let current_lang = self.config.language;
                                self.config = preset;
                                self.config.language = current_lang;
                                let _ = self.config.save();
                                self.status_message = format!("Printer: {}", self.config.clean_name());
                            }
                        }
                        ui.separator();
                        if ui.button(Tr::configure_printers(lang)).clicked() {
                            self.show_setup_wizard = true;
                        }
                    });
            });
        });

        // 3. CENTER ZONE: Pipeline Tabs [ Подготовка | Калибровка | Предпросмотр ]
        // Strictly centered on panel_rect with collision protection
        let capsule_w = 264.0;
        let capsule_h = 26.0;

        let ideal_center_x = panel_rect.center().x;
        let min_center_x = left_rect.right() + 16.0 + capsule_w * 0.5;
        let max_center_x = right_rect.left() - 16.0 - capsule_w * 0.5;
        let center_x = if min_center_x < max_center_x {
            ideal_center_x.clamp(min_center_x, max_center_x)
        } else {
            ideal_center_x
        };

        let capsule_rect = Rect::from_center_size(
            pos2(center_x, panel_rect.center().y),
            vec2(capsule_w, capsule_h),
        );

        // Draw monolithic dark capsule track
        ui.painter().rect_filled(
            capsule_rect,
            6.0,
            Color32::from_rgb(20, 20, 22), // #141416
        );
        ui.painter().rect_stroke(
            capsule_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(39, 39, 42)), // 1px #27272a
        );

        let inner_rect = capsule_rect.shrink(2.0);
        let tab_width = inner_rect.width() / 3.0;

        let tabs = [
            (StepTab::Prepare, Tr::tab_prepare(lang)),
            (StepTab::Calibrate, Tr::tab_calibrate(lang)),
            (StepTab::Preview, Tr::tab_preview(lang)),
        ];

        for (i, (tab, title)) in tabs.into_iter().enumerate() {
            let item_rect = Rect::from_min_size(
                pos2(inner_rect.left() + i as f32 * tab_width, inner_rect.top()),
                vec2(tab_width, inner_rect.height()),
            );

            let is_active = self.active_tab == tab;
            let resp = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_active {
                // Active sliding thumb
                ui.painter().rect_filled(
                    item_rect,
                    4.0,
                    Color32::from_rgb(39, 39, 42), // #27272a
                );
                ui.painter().rect_stroke(
                    item_rect,
                    4.0,
                    Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)), // #3f3f46
                );
            } else if resp.hovered() {
                ui.painter().rect_filled(
                    item_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                );
            }

            let text_color = if is_active {
                Color32::from_rgb(244, 244, 245)
            } else {
                Color32::from_rgb(161, 161, 170)
            };

            ui.painter().text(
                item_rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                FontId::new(11.5, if is_active { FontFamily::Name("Inter-SemiBold".into()) } else { FontFamily::Proportional }),
                text_color,
            );

            if resp.clicked() {
                self.active_tab = tab;
                if tab == StepTab::Calibrate && self.calibration.original_texture.is_some() {
                    self.mode = AppMode::Calibration;
                } else {
                    self.mode = AppMode::BedView;
                }
            }
        }
    }

    /// Left Sidebar (44px): Strict neutral toolbar with centered vector glyphs
    fn render_left_sidebar(&mut self, ui: &mut Ui, ctx: &Context) {
        let is_in_calibration = self.active_tab == StepTab::Calibrate;
        let lang = self.config.language;

        ui.vertical_centered(|ui| {
            ui.add_space(8.0);

            let tools = [
                (ToolId::Select, "↖", Tr::tool_select(lang)),
                (ToolId::Text, "T", Tr::tool_text(lang)),
                (ToolId::Rectangle, "▭", Tr::tool_rect(lang)),
                (ToolId::Circle, "○", Tr::tool_circle(lang)),
                (ToolId::Star, "☆", Tr::tool_star(lang)),
                (ToolId::TestGrid, "⊞", Tr::tool_grid(lang)),
            ];

            for (tool, icon, tip) in tools {
                let is_sel = self.active_tool == tool;
                let bg = if is_sel {
                    Color32::from_rgb(39, 39, 42) // #27272a
                } else {
                    Color32::TRANSPARENT
                };
                let fg = if is_in_calibration && tool != ToolId::Select {
                    Color32::from_rgb(82, 82, 91) // disabled muted
                } else if is_sel {
                    Color32::from_rgb(244, 244, 245)
                } else {
                    Color32::from_rgb(161, 161, 170)
                };
                let stroke = if is_sel {
                    Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70))
                } else {
                    Stroke::NONE
                };

                let btn = egui::Button::new(
                    RichText::new(icon)
                        .font(FontId::new(15.0, FontFamily::Proportional))
                        .color(fg),
                )
                .fill(bg)
                .stroke(stroke)
                .rounding(Rounding::same(6.0));

                let resp = ui.add_enabled(!is_in_calibration || tool == ToolId::Select, btn);
                if resp.clicked() {
                    self.active_tool = tool;
                }
                resp.on_hover_text(tip);
                ui.add_space(4.0);
            }

            ui.add_space(6.0);
            // Strict relative coordinate calculation (no hardcoded screen x coordinates!)
            let bar_rect = ui.max_rect();
            let y_line = ui.cursor().top();
            ui.painter().line_segment(
                [pos2(bar_rect.left() + 6.0, y_line), pos2(bar_rect.right() - 6.0, y_line)],
                Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)),
            );
            ui.add_space(6.0);

            // Import SVG Button: Monochromatic vector glyph ⤓
            let btn_svg = egui::Button::new(
                RichText::new("⤓")
                    .font(FontId::new(15.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(161, 161, 170)),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0));

            let resp_svg = ui.add_sized([32.0, 32.0], btn_svg);
            if resp_svg.clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).pick_file() {
                    let _ = self.vector_doc.load_svg(&path);
                    self.vector_doc.center_on_safe_area(&self.config);
                    self.status_message = match self.config.language {
                        Language::Russian => format!("Загружен SVG: {}", self.vector_doc.name),
                        Language::English => format!("Loaded SVG: {}", self.vector_doc.name),
                    };
                }
            }
            resp_svg.on_hover_text(Tr::import_svg(self.config.language));

            ui.add_space(4.0);

            // Load Bed Photo Button: Monochromatic vector glyph ⊡
            let btn_cam = egui::Button::new(
                RichText::new("⊡")
                    .font(FontId::new(15.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(161, 161, 170)),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0));

            let resp_cam = ui.add_sized([32.0, 32.0], btn_cam);
            if resp_cam.clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("Images", &["jpg", "png", "webp"]).pick_file() {
                    let _ = self.calibration.load_image_from_path(&path, ctx);
                    self.active_tab = StepTab::Calibrate;
                    self.mode = AppMode::Calibration;
                }
            }
            resp_cam.on_hover_text(Tr::load_bed_photo(self.config.language));
        });
    }

    /// Right Sidebar (270px): Cards synchronized with active Slicer Tab
    fn render_right_sidebar(&mut self, ui: &mut Ui, ctx: &Context) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.active_tab {
                StepTab::Prepare => self.render_prepare_cards(ui),
                StepTab::Calibrate => self.render_calibration_cards(ui, ctx),
                StepTab::Preview => self.render_preview_cards(ui),
            }
        });
    }

    /// Synchronize text editor controls from the currently selected item if it has text metadata
    pub fn sync_text_controls_from_selection(&mut self) {
        if let Some(td) = self.vector_doc.selected_text_data() {
            self.text_input = td.text.clone();
            self.text_size_mm = td.font_size_mm;
            self.font_style = td.style;
            self.humanizer_strength = td.humanizer_strength;
            self.humanizer_enabled = td.humanizer_strength > 0.001;
        }
    }

    fn render_prepare_cards(&mut self, ui: &mut Ui) {
        let lang = self.config.language;

        // --- Card 1: Tool Parameters ---
        self.render_card(ui, Tr::card_creation(lang), |ui, app| {
            match app.active_tool {
                ToolId::Select => {
                    let desc = match lang {
                        Language::Russian => "Выберите инструмент слева или кликните по объекту на столе для редактирования.",
                        Language::English => "Select a tool on the left or click an object on the bed to edit.",
                    };
                    ui.label(
                        RichText::new(desc)
                            .small()
                            .color(Color32::from_rgb(161, 161, 170)),
                    );
                }
                ToolId::Text => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("●").color(Color32::from_rgb(16, 185, 129))); // emerald live dot
                        let live_hint = match lang {
                            Language::Russian => "Редактирование на лету (WYSIWYG)",
                            Language::English => "Live WYSIWYG editing",
                        };
                        ui.label(RichText::new(live_hint).small().color(Color32::from_rgb(161, 161, 170)));
                    });
                    ui.add_space(4.0);

                    let mut text_changed = false;
                    let mm_suf = format!(" {}", Tr::unit_mm(lang));
                    ui.label(RichText::new(Tr::text_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    let text_hint = match lang {
                        Language::Russian => "Введите текст (поддерживается русский язык и перенос строк)...",
                        Language::English => "Enter text (multiline supported)...",
                    };
                    let text_resp = ui.add(
                        egui::TextEdit::multiline(&mut app.text_input)
                            .desired_rows(2)
                            .desired_width(ui.available_width())
                            .hint_text(text_hint)
                    );
                    if text_resp.changed() {
                        text_changed = true;
                    }

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::height_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        if ui.add(egui::DragValue::new(&mut app.text_size_mm).speed(0.5).range(3.0..=120.0).suffix(&mm_suf)).changed() {
                            text_changed = true;
                        }
                    });

                    ui.add_space(4.0);
                    // Font style selection: GOST Technical vs Handwritten Cursive
                    ui.label(RichText::new(Tr::font_style(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.horizontal(|ui| {
                        let is_gost = app.font_style == FontStyle::TechnicalGost;
                        if ui.selectable_label(is_gost, Tr::font_gost(lang)).clicked() && !is_gost {
                            app.font_style = FontStyle::TechnicalGost;
                            text_changed = true;
                        }
                        let is_cursive = app.font_style == FontStyle::Cursive;
                        if ui.selectable_label(is_cursive, Tr::font_cursive(lang)).clicked() && !is_cursive {
                            app.font_style = FontStyle::Cursive;
                            text_changed = true;
                        }
                    });

                    ui.add_space(4.0);
                    // Organic Hand-drawn Humanizer
                    if ui.checkbox(&mut app.humanizer_enabled, Tr::humanizer_label(lang)).changed() {
                        text_changed = true;
                    }
                    if app.humanizer_enabled {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::humanizer_strength(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            let mut pct = (app.humanizer_strength * 100.0) as i32;
                            if ui.add(egui::Slider::new(&mut pct, 5..=100).suffix("%")).changed() {
                                app.humanizer_strength = pct as f32 / 100.0;
                                text_changed = true;
                            }
                        });
                    }

                    // REAL-TIME INSTANT UPDATE:
                    let h_str = if app.humanizer_enabled { app.humanizer_strength } else { 0.0 };
                    if text_changed {
                        if app.vector_doc.selected_text_data().is_some() {
                            app.vector_doc.update_selected_text(&app.text_input, app.text_size_mm, app.font_style, h_str);
                        } else {
                            let (cx, cy) = app.config.safe_pen_center();
                            app.vector_doc.add_text_at(&app.text_input, app.text_size_mm, pos2(cx, cy), app.font_style, h_str);
                        }
                    }

                    ui.add_space(6.0);
                    let add_label = Tr::new_text_block_btn(lang);
                    let btn = egui::Button::new(RichText::new(add_label).color(Color32::from_rgb(244, 244, 245)))
                        .fill(Color32::from_rgb(39, 39, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(6.0));
                    let new_text_tip = match lang {
                        Language::Russian => "Создать новый независимый текстовый блок",
                        Language::English => "Create a new independent text block",
                    };
                    if ui.add_sized([ui.available_width(), 26.0], btn).on_hover_text(new_text_tip).clicked() {
                        let (cx, cy) = app.config.safe_pen_center();
                        let offset_y = if let Some(bb) = app.vector_doc.bounding_box() {
                            (bb.max.y + 15.0).min(app.config.bed_height - 20.0)
                        } else {
                            cy
                        };
                        app.vector_doc.add_text_at(&app.text_input, app.text_size_mm, pos2(cx, offset_y), app.font_style, h_str);
                        app.status_message = match lang {
                            Language::Russian => format!("Новый текст: \"{}\"", app.text_input),
                            Language::English => format!("New text: \"{}\"", app.text_input),
                        };
                    }
                }
                ToolId::Rectangle => {
                    let mm_suf = format!(" {}", Tr::unit_mm(lang));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::width(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.rect_w).speed(1.0).range(5.0..=300.0).suffix(&mm_suf));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::length(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.rect_h).speed(1.0).range(5.0..=300.0).suffix(&mm_suf));
                    });

                    ui.add_space(4.0);
                    // Hatch Infill
                    ui.checkbox(&mut app.hatch_enabled, Tr::hatch_infill(lang));
                    if app.hatch_enabled {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_spacing(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_spacing, 0.4..=5.0).suffix(&mm_suf));
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_angle(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_angle, 0.0..=90.0).suffix("°"));
                        });
                        ui.checkbox(&mut app.hatch_cross, Tr::cross_hatch(lang));
                    }

                    ui.add_space(6.0);
                    let add_label = Tr::add_rect_btn(lang);
                    let btn = egui::Button::new(RichText::new(add_label).color(Color32::from_rgb(244, 244, 245)))
                        .fill(Color32::from_rgb(39, 39, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(6.0));
                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                        app.vector_doc.add_rectangle_advanced(
                            app.rect_w,
                            app.rect_h,
                            app.hatch_enabled,
                            app.hatch_angle,
                            app.hatch_spacing,
                            app.hatch_cross,
                        );
                        app.vector_doc.center_on_safe_area(&app.config);
                        app.active_tool = ToolId::Select;
                        app.status_message = match lang {
                            Language::Russian => "Прямоугольник добавлен.".to_string(),
                            Language::English => "Rectangle added.".to_string(),
                        };
                    }
                }
                ToolId::Circle => {
                    let mm_suf = format!(" {}", Tr::unit_mm(lang));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::radius(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.circle_r).speed(1.0).range(2.0..=150.0).suffix(&mm_suf));
                    });

                    ui.add_space(4.0);
                    // Hatch Infill
                    ui.checkbox(&mut app.hatch_enabled, Tr::hatch_infill(lang));
                    if app.hatch_enabled {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_spacing(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_spacing, 0.4..=5.0).suffix(&mm_suf));
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_angle(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_angle, 0.0..=90.0).suffix("°"));
                        });
                        ui.checkbox(&mut app.hatch_cross, Tr::cross_hatch(lang));
                    }

                    ui.add_space(6.0);
                    let add_label = Tr::add_circle_btn(lang);
                    let btn = egui::Button::new(RichText::new(add_label).color(Color32::from_rgb(244, 244, 245)))
                        .fill(Color32::from_rgb(39, 39, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(6.0));
                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                        app.vector_doc.add_circle_advanced(
                            app.circle_r,
                            48,
                            app.hatch_enabled,
                            app.hatch_angle,
                            app.hatch_spacing,
                            app.hatch_cross,
                        );
                        app.vector_doc.center_on_safe_area(&app.config);
                        app.active_tool = ToolId::Select;
                        app.status_message = match lang {
                            Language::Russian => "Круг добавлен.".to_string(),
                            Language::English => "Circle added.".to_string(),
                        };
                    }
                }
                ToolId::Star => {
                    let mm_suf = format!(" {}", Tr::unit_mm(lang));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::rays(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.star_pts).range(3..=16));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::outer_radius(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.star_r_out).speed(1.0).range(5.0..=120.0).suffix(&mm_suf));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::inner_radius(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.star_r_in).speed(1.0).range(2.0..=100.0).suffix(&mm_suf));
                    });

                    ui.add_space(4.0);
                    // Hatch Infill
                    ui.checkbox(&mut app.hatch_enabled, Tr::hatch_infill(lang));
                    if app.hatch_enabled {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_spacing(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_spacing, 0.4..=5.0).suffix(&mm_suf));
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Tr::hatch_angle(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                            ui.add(egui::Slider::new(&mut app.hatch_angle, 0.0..=90.0).suffix("°"));
                        });
                        ui.checkbox(&mut app.hatch_cross, Tr::cross_hatch(lang));
                    }

                    ui.add_space(6.0);
                    let add_label = Tr::add_star_btn(lang);
                    let btn = egui::Button::new(RichText::new(add_label).color(Color32::from_rgb(244, 244, 245)))
                        .fill(Color32::from_rgb(39, 39, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(6.0));
                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                        app.vector_doc.add_star_advanced(
                            app.star_r_out,
                            app.star_r_in,
                            app.star_pts,
                            app.hatch_enabled,
                            app.hatch_angle,
                            app.hatch_spacing,
                            app.hatch_cross,
                        );
                        app.vector_doc.center_on_safe_area(&app.config);
                        app.active_tool = ToolId::Select;
                        app.status_message = match lang {
                            Language::Russian => "Звезда добавлена.".to_string(),
                            Language::English => "Star added.".to_string(),
                        };
                    }
                }
                ToolId::TestGrid => {
                    let mm_suf = format!(" {}", Tr::unit_mm(lang));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::width(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.test_size).speed(1.0).range(10.0..=220.0).suffix(&mm_suf));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::step_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::DragValue::new(&mut app.test_step).speed(1.0).range(2.0..=50.0).suffix(&mm_suf));
                    });
                    ui.add_space(4.0);
                    let add_label = Tr::add_grid_btn(lang);
                    let btn = egui::Button::new(RichText::new(add_label).color(Color32::from_rgb(244, 244, 245)))
                        .fill(Color32::from_rgb(39, 39, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(6.0));
                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                        app.vector_doc.add_test_pattern(app.test_size, app.test_step);
                        app.vector_doc.center_on_safe_area(&app.config);
                        app.active_tool = ToolId::Select;
                        app.status_message = match lang {
                            Language::Russian => "Калибровочная сетка добавлена.".to_string(),
                            Language::English => "Calibration grid added.".to_string(),
                        };
                    }
                }
            }
        });

        ui.add_space(8.0);

        // --- Card 2: Layers & Objects on Bed (Multi-object scene graph) ---
        if !self.vector_doc.is_empty() {
            let layers_title = Tr::layers_title(lang);
            self.render_card(ui, layers_title, |ui, app| {
                let mut to_remove = None;
                let mut to_select = None;
                let selected_idx = app.vector_doc.selected_index;

                egui::ScrollArea::vertical()
                    .max_height(130.0)
                    .show(ui, |ui| {
                        for (idx, item) in app.vector_doc.items.iter().enumerate() {
                            let is_selected = selected_idx == Some(idx);
                            ui.horizontal(|ui| {
                                let label_text = format!("{}. {}", idx + 1, item.name);
                                let resp = ui.selectable_label(is_selected, RichText::new(label_text).small());
                                if resp.clicked() {
                                    to_select = Some(idx);
                                }
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let btn_del = egui::Button::new(RichText::new("🗑").small().color(Color32::from_rgb(239, 68, 68)))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE);
                                    if ui.add(btn_del).on_hover_text(Tr::delete_object_tip(lang)).clicked() {
                                        to_remove = Some(idx);
                                    }
                                });
                            });
                        }
                    });

                if let Some(idx) = to_remove {
                    app.vector_doc.remove_item(idx);
                } else if let Some(idx) = to_select {
                    app.vector_doc.select_item(idx);
                    app.sync_text_controls_from_selection();
                }

                if app.vector_doc.items.len() > 1 {
                    ui.add_space(6.0);
                    let merge_btn = egui::Button::new(
                        RichText::new(Tr::merge_all_btn(lang))
                            .small()
                            .color(Color32::from_rgb(244, 244, 245)),
                    )
                    .fill(Color32::from_rgb(39, 39, 42))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)));

                    if ui.add_sized([ui.available_width(), 24.0], merge_btn).on_hover_text(Tr::merge_tip(lang)).clicked() {
                        app.vector_doc.merge_all();
                    }
                }
            });
            ui.add_space(8.0);
        }

        // --- Card 3: Selected Object Properties ---
        if !self.vector_doc.is_empty() {
            let card_name = if self.vector_doc.items.len() > 1 {
                if let Some(item) = self.vector_doc.selected_item() {
                    match lang {
                        Language::Russian => format!("ПАРАМЕТРЫ: {}", item.name),
                        Language::English => format!("PROPERTIES: {}", item.name),
                    }
                } else {
                    Tr::card_transform(lang).to_string()
                }
            } else {
                Tr::card_transform(lang).to_string()
            };

            self.render_card(ui, &card_name, |ui, app| {
                let mm_suf = format!(" {}", Tr::unit_mm(lang));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&app.vector_doc.name).font(FontId::new(13.0, FontFamily::Name("Inter-SemiBold".into()))).color(Color32::from_rgb(244, 244, 245)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let btn_del = egui::Button::new(RichText::new(Tr::clear(lang)).small().color(Color32::from_rgb(239, 68, 68)))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);
                        if ui.add(btn_del).clicked() {
                            if let Some(idx) = app.vector_doc.selected_index {
                                app.vector_doc.remove_item(idx);
                            } else {
                                app.vector_doc.clear();
                            }
                        }
                    });
                });

                // Live real-time typography editing if active object is Text
                if app.vector_doc.selected_text_data().is_some() {
                    ui.add_space(4.0);
                    let mut txt_changed = false;
                    let edit_txt_label = match lang {
                        Language::Russian => "Редактировать текст:",
                        Language::English => "Edit text:",
                    };
                    ui.label(RichText::new(edit_txt_label).small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(
                        egui::TextEdit::multiline(&mut app.text_input)
                            .desired_rows(2)
                            .desired_width(ui.available_width())
                    ).changed() {
                        txt_changed = true;
                    }

                    ui.horizontal(|ui| {
                        let is_gost = app.font_style == FontStyle::TechnicalGost;
                        if ui.selectable_label(is_gost, Tr::font_gost(lang)).clicked() && !is_gost {
                            app.font_style = FontStyle::TechnicalGost;
                            txt_changed = true;
                        }
                        let is_cursive = app.font_style == FontStyle::Cursive;
                        if ui.selectable_label(is_cursive, Tr::font_cursive(lang)).clicked() && !is_cursive {
                            app.font_style = FontStyle::Cursive;
                            txt_changed = true;
                        }
                    });

                    if txt_changed {
                        let h_str = if app.humanizer_enabled { app.humanizer_strength } else { 0.0 };
                        app.vector_doc.update_selected_text(&app.text_input, app.text_size_mm, app.font_style, h_str);
                    }
                    ui.add_space(4.0);
                }

                let stat_desc = match lang {
                    Language::Russian => format!(
                        "Контуров: {}  |  Длина пути: {:.1} мм",
                        app.vector_doc.raw_polylines.len(),
                        app.vector_doc.total_length_mm()
                    ),
                    Language::English => format!(
                        "Contours: {}  |  Path length: {:.1} mm",
                        app.vector_doc.raw_polylines.len(),
                        app.vector_doc.total_length_mm()
                    ),
                };
                ui.label(
                    RichText::new(stat_desc)
                        .small()
                        .color(Color32::from_rgb(161, 161, 170)),
                );

                ui.add_space(6.0);

                // Physical Dimensions (with Aspect Ratio Lock indication)
                let (mut cur_w, mut cur_h) = app.vector_doc.get_dimensions_mm();
                ui.horizontal(|ui| {
                    let lock_text = Tr::lock_aspect_ratio_label(lang, app.vector_doc.lock_aspect_ratio);
                    if ui.selectable_label(app.vector_doc.lock_aspect_ratio, RichText::new(lock_text).small()).clicked() {
                        app.vector_doc.lock_aspect_ratio = !app.vector_doc.lock_aspect_ratio;
                        app.vector_doc.sync_to_selected();
                    }
                });

                egui::Grid::new("obj_dim_grid").num_columns(4).spacing([6.0, 6.0]).show(ui, |ui| {
                    ui.label(RichText::new(Tr::width(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut cur_w).speed(0.5).range(1.0..=1000.0).suffix(&mm_suf)).changed() {
                        app.vector_doc.set_width_mm(cur_w);
                    }
                    ui.label(RichText::new(Tr::height_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut cur_h).speed(0.5).range(1.0..=1000.0).suffix(&mm_suf)).changed() {
                        app.vector_doc.set_height_mm(cur_h);
                    }
                    ui.end_row();

                    ui.label(RichText::new("X:").small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut app.vector_doc.position.x).speed(0.5).range(0.0..=app.config.bed_width).suffix(&mm_suf)).changed() {
                        app.vector_doc.sync_to_selected();
                    }
                    ui.label(RichText::new("Y:").small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut app.vector_doc.position.y).speed(0.5).range(0.0..=app.config.bed_height).suffix(&mm_suf)).changed() {
                        app.vector_doc.sync_to_selected();
                    }
                    ui.end_row();

                    ui.label(RichText::new(Tr::scale_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut app.vector_doc.scale).speed(0.02).range(0.05..=10.0)).changed() {
                        app.vector_doc.sync_to_selected();
                    }
                    ui.label(RichText::new(Tr::angle_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    if ui.add(egui::DragValue::new(&mut app.vector_doc.rotation_deg).speed(1.0).range(-180.0..=180.0).suffix("°")).changed() {
                        app.vector_doc.sync_to_selected();
                    }
                    ui.end_row();
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(Tr::center_on_bed(lang)).clicked() {
                        app.vector_doc.center_on_safe_area(&app.config);
                    }
                    if ui.button(Tr::fit_to_bed(lang)).clicked() {
                        app.vector_doc.fit_to_safe_area(&app.config, 20.0);
                    }
                    if ui.button("90°").clicked() {
                        app.vector_doc.rotation_deg = (app.vector_doc.rotation_deg + 90.0) % 360.0;
                        app.vector_doc.sync_to_selected();
                    }
                });
            });
        }

        ui.add_space(8.0);

        // --- Card: Quick Templates (shown when canvas is empty to eliminate blank space) ---
        if self.vector_doc.is_empty() {
            self.render_card(ui, Tr::card_quick_templates(lang), |ui, app| {
                ui.label(RichText::new(Tr::quick_templates_hint(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.add_space(4.0);

                let templates = Tr::quick_template_items(lang);
                let btn_w = (ui.available_width() - 6.0) * 0.5;
                egui::Grid::new("templates_2x2_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        for (i, (name, w, h)) in templates.iter().enumerate() {
                            let btn = egui::Button::new(RichText::new(*name).small().color(Color32::from_rgb(244, 244, 245)))
                                .fill(Color32::from_rgb(39, 39, 42))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                                .rounding(Rounding::same(6.0));
                            if ui.add_sized([btn_w, 26.0], btn).clicked() {
                                let (cx, cy) = app.config.safe_pen_center();
                                app.vector_doc.add_rectangle_at(*w, *h, egui::pos2(cx, cy), false, 45.0, 1.5, false);
                                app.active_tool = ToolId::Select;
                                app.status_message = match lang {
                                    Language::Russian => format!("Шаблон: {}", name),
                                    Language::English => format!("Template: {}", name),
                                };
                            }
                            if i % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
            });
            ui.add_space(8.0);
        }

        // --- Card: Plotter Workspace & Offset Calibration ---
        self.render_card(ui, Tr::card_workspace_offset(lang), |ui, app| {
            let (max_draw_w, max_draw_h) = app.config.safe_pen_dimensions();
            ui.label(RichText::new(Tr::printable_area(lang, max_draw_w, max_draw_h)).small().color(Color32::from_rgb(244, 244, 245)));

            let offset_hint_x = Tr::offset_hint_x(lang, app.config.toolhead_offset_x);
            let offset_hint_y = Tr::offset_hint_y(lang, app.config.toolhead_offset_y);

            ui.add_space(2.0);
            ui.label(RichText::new(Tr::offset_relative_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));

            let mm_suf = format!(" {}", Tr::unit_mm(lang));
            egui::Grid::new("offset_quick_grid").num_columns(3).spacing([6.0, 4.0]).show(ui, |ui| {
                ui.label("ΔX:");
                if ui.add(egui::DragValue::new(&mut app.config.toolhead_offset_x).speed(0.5).range(-250.0..=250.0).suffix(&mm_suf)).changed() {
                    let _ = app.config.save();
                }
                ui.label(RichText::new(format!("({})", offset_hint_x)).small().color(Color32::from_rgb(113, 113, 122)));
                ui.end_row();

                ui.label("ΔY:");
                if ui.add(egui::DragValue::new(&mut app.config.toolhead_offset_y).speed(0.5).range(-250.0..=250.0).suffix(&mm_suf)).changed() {
                    let _ = app.config.save();
                }
                ui.label(RichText::new(format!("({})", offset_hint_y)).small().color(Color32::from_rgb(113, 113, 122)));
                ui.end_row();
            });

            ui.add_space(4.0);
            let mech_desc = match &app.config.pen_mechanism {
                crate::config::PenMechanism::ZAxis { z_safe, z_draw, .. } => {
                    Tr::z_axis_desc(lang, *z_safe, *z_draw)
                }
                crate::config::PenMechanism::KlipperMacros { .. } => "Klipper Macros (PEN_UP/DOWN)".to_string(),
                crate::config::PenMechanism::Servo { pin, .. } => Tr::servo_desc(lang, *pin),
            };
            ui.label(RichText::new(format!("{} {}", Tr::lift_label(lang), mech_desc)).small().color(Color32::from_rgb(161, 161, 170)));

            let speed_unit = Tr::unit_speed(lang);
            let speed_desc = format!("G0: {:.0} | G1: {:.0} {}", app.config.feedrate_travel, app.config.feedrate_draw, speed_unit);
            ui.label(RichText::new(speed_desc).small().color(Color32::from_rgb(161, 161, 170)));

            ui.add_space(6.0);
            let btn_cfg = egui::Button::new(RichText::new(Tr::configure_printer_btn(lang)).small().color(Color32::from_rgb(228, 228, 231)))
                .fill(Color32::from_rgb(39, 39, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                .rounding(Rounding::same(6.0));
            if ui.add_sized([ui.available_width(), 26.0], btn_cfg).clicked() {
                app.show_setup_wizard = true;
            }
        });
        ui.add_space(8.0);

        // --- Card: Pen Tip Diameter & Stroke Thickness ---
        self.render_card(ui, Tr::card_pen_tip(lang), |ui, app| {
            ui.label(RichText::new(Tr::pen_tip_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
            let mm_suf = format!(" {}", Tr::unit_mm(lang));

            ui.horizontal(|ui| {
                if ui.add(
                    egui::DragValue::new(&mut app.config.pen_tip_diameter_mm)
                        .speed(0.05)
                        .range(0.1..=5.0)
                        .suffix(&mm_suf),
                ).changed() {
                    let _ = app.config.save();
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let presets = Tr::pen_presets(lang);
                for (name, val) in presets {
                    let is_active = (app.config.pen_tip_diameter_mm - val).abs() < 0.05;
                    let fill = if is_active {
                        Color32::from_rgb(59, 130, 246) // #3b82f6 active
                    } else {
                        Color32::from_rgb(39, 39, 42)
                    };
                    let btn = egui::Button::new(RichText::new(name).small().color(Color32::from_rgb(244, 244, 245)))
                        .fill(fill)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                        .rounding(Rounding::same(4.0));
                    if ui.add(btn).clicked() {
                        app.config.pen_tip_diameter_mm = val;
                        let _ = app.config.save();
                    }
                }
            });
        });
    }

    fn render_calibration_cards(&mut self, ui: &mut Ui, ctx: &Context) {
        let lang = self.config.language;
        self.render_card(ui, Tr::card_calibration(lang), |ui, app| {
            let btn_load = egui::Button::new(RichText::new(Tr::load_bed_photo_btn(lang)).color(Color32::from_rgb(244, 244, 245)))
                .fill(Color32::from_rgb(39, 39, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                .rounding(Rounding::same(6.0));

            if ui.add_sized([ui.available_width(), 30.0], btn_load).clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("Images", &["jpg", "png", "webp"]).pick_file() {
                    let _ = app.calibration.load_image_from_path(&path, ctx);
                    app.mode = AppMode::Calibration;
                }
            }

            if app.calibration.original_texture.is_some() {
                ui.add_space(6.0);
                ui.label(RichText::new(Tr::calibration_target_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.horizontal(|ui| {
                    let is_bed = app.calibration.target == CalibrationTarget::FullBed;
                    if ui.selectable_label(is_bed, Tr::target_full_bed(lang)).clicked() {
                        app.calibration.target = CalibrationTarget::FullBed;
                    }
                    let is_a4 = app.calibration.target == CalibrationTarget::A4Sheet;
                    if ui.selectable_label(is_a4, Tr::target_a4(lang)).clicked() {
                        app.calibration.target = CalibrationTarget::A4Sheet;
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button(Tr::reset_points(lang)).clicked() {
                        app.calibration.reset_corners();
                    }
                });

                ui.add_space(8.0);
                let btn_rectify = egui::Button::new(
                    RichText::new(Tr::rectify_perspective(lang))
                        .color(Color32::from_rgb(244, 244, 245)),
                )
                .fill(Color32::from_rgb(59, 130, 246)) // #3b82f6
                .stroke(Stroke::NONE)
                .rounding(Rounding::same(6.0));

                if ui.add_sized([ui.available_width(), 32.0], btn_rectify).clicked() {
                    let bw = app.config.bed_width as f64;
                    let bh = app.config.bed_height as f64;
                    if let Ok(()) = app.calibration.rectify(ctx, bw, bh) {
                        app.mode = AppMode::BedView;
                        app.status_message = match lang {
                            Language::Russian => "Перспектива выровнена!".to_string(),
                            Language::English => "Perspective rectified!".to_string(),
                        };
                    }
                }

                if app.calibration.is_rectified {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Tr::opacity(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        ui.add(egui::Slider::new(&mut app.calibration.photo_opacity, 0.0..=1.0));
                    });
                }
            } else {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(Tr::calibrate_hint(lang))
                        .small()
                        .color(Color32::from_rgb(113, 113, 122)),
                );
            }
        });
    }

    pub fn update_opt_cache_if_needed(&mut self) {
        let is_valid = self.cached_opt_paths.is_some()
            && self.cached_opt_version == self.vector_doc.version
            && self.doc_version == self.vector_doc.version;
        if !is_valid {
            let raw_polylines = self.vector_doc.get_transformed_polylines();
            let opt = if self.optimize_paths {
                PathOptimizer::optimize(raw_polylines)
            } else {
                raw_polylines
            };
            self.cached_opt_paths = Some(opt);
            self.cached_opt_version = self.vector_doc.version;
            self.doc_version = self.vector_doc.version;
        }
    }

    /// Tab 3: Предпросмотр (Preview, ETA, Simulation & Slicer Plotter params)
    fn render_preview_cards(&mut self, ui: &mut Ui) {
        self.update_opt_cache_if_needed();
        let raw_polylines = self.vector_doc.get_transformed_polylines();
        let opt_polylines = self.cached_opt_paths.clone().unwrap_or_default();

        let draw_len_mm = self.vector_doc.total_length_mm();
        let raw_travel_mm = PathOptimizer::travel_distance(&raw_polylines);
        let opt_travel_mm = PathOptimizer::travel_distance(&opt_polylines);
        let savings_pct = if raw_travel_mm > 1.0 {
            ((raw_travel_mm - opt_travel_mm) / raw_travel_mm * 100.0).max(0.0)
        } else {
            0.0
        };

        let speed_draw_mms = (self.config.feedrate_draw / 60.0).max(1.0);
        let speed_travel_mms = (self.config.feedrate_travel / 60.0).max(1.0);
        let active_travel_mm = if self.optimize_paths { opt_travel_mm } else { raw_travel_mm };
        let passes = self.multipass_count.max(1) as f32;
        let total_draw_mm = (draw_len_mm * passes).max(0.0);
        let active_travel_mm = active_travel_mm.max(0.0);
        let est_time_sec = ((draw_len_mm / speed_draw_mms) + (active_travel_mm / speed_travel_mms)) * passes;
        let est_min = (est_time_sec / 60.0) as u32;
        let est_sec = (est_time_sec % 60.0) as u32;

        // --- Card 1: G-code Simulation & ETA Metrics ---
        let total_segs = GcodePreview::total_segments(&raw_polylines);
        let lang = self.config.language;
        self.render_card(ui, Tr::card_simulation(lang), |ui, app| {
            let eta_title = Tr::est_time(lang, est_min, est_sec);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(eta_title)
                        .font(FontId::new(13.0, FontFamily::Name("Inter-SemiBold".into())))
                        .color(Color32::from_rgb(244, 244, 245)),
                );
            });

            let stats_line = Tr::preview_stats(lang, total_draw_mm, total_segs, active_travel_mm);
            ui.label(
                RichText::new(stats_line)
                    .small()
                    .color(Color32::from_rgb(161, 161, 170)),
            );

            if app.optimize_paths && savings_pct > 1.0 {
                let tsp_line = Tr::savings_tsp(lang, savings_pct);
                ui.label(
                    RichText::new(tsp_line)
                        .small()
                        .color(Color32::from_rgb(16, 185, 129)),
                );
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(Tr::progress_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.add(egui::Slider::new(&mut app.preview.progress, 0.0..=1.0).show_value(false));
                ui.label(format!("{:.0}%", app.preview.progress * 100.0));
            });

            ui.horizontal(|ui| {
                let play_text = Tr::play_btn(lang, app.preview.is_playing);
                if ui.button(play_text).clicked() {
                    app.preview.is_playing = !app.preview.is_playing;
                    if app.preview.progress >= 1.0 {
                        app.preview.progress = 0.0;
                    }
                }
                if ui.button("100%").clicked() {
                    app.preview.progress = 1.0;
                    app.preview.is_playing = false;
                }
            });
        });

        ui.add_space(8.0);

        // --- Card 2: Plotter & Generator Parameters ---
        let lang = self.config.language;
        self.render_card(ui, Tr::card_plotter(lang), |ui, app| {
            if ui.checkbox(&mut app.optimize_paths, Tr::optimize_paths(lang)).changed() {
                app.doc_version = app.doc_version.wrapping_add(1);
            }
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new(Tr::multipass(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.add(egui::DragValue::new(&mut app.multipass_count).range(1..=5));
            });
            ui.add_space(4.0);

            let speed_suf = format!(" {}", Tr::unit_speed(lang));
            let mm_suf = format!(" {}", Tr::unit_mm(lang));

            ui.horizontal(|ui| {
                ui.label(RichText::new(Tr::feedrate_travel_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.add(egui::DragValue::new(&mut app.config.feedrate_travel).speed(50.0).range(100.0..=15000.0).suffix(&speed_suf));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(Tr::feedrate_draw_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                ui.add(egui::DragValue::new(&mut app.config.feedrate_draw).speed(50.0).range(100.0..=10000.0).suffix(&speed_suf));
            });

            ui.separator();
            ui.label(RichText::new(Tr::toolhead_offset_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
            egui::Grid::new("offset_grid").num_columns(4).spacing([6.0, 6.0]).show(ui, |ui| {
                ui.label("ΔX:");
                ui.add(egui::DragValue::new(&mut app.config.toolhead_offset_x).speed(0.5).range(-200.0..=200.0).suffix(&mm_suf));
                ui.label("ΔY:");
                ui.add(egui::DragValue::new(&mut app.config.toolhead_offset_y).speed(0.5).range(-200.0..=200.0).suffix(&mm_suf));
            });

            ui.add_space(8.0);
            let is_ready = !app.vector_doc.is_empty();
            let (btn_fill, btn_fg) = if is_ready {
                (Color32::from_rgb(16, 185, 129), Color32::from_rgb(18, 18, 20))
            } else {
                (Color32::from_rgb(39, 39, 42), Color32::from_rgb(113, 113, 122))
            };

            let btn = egui::Button::new(
                RichText::new(Tr::export_gcode(lang))
                    .color(btn_fg),
            )
            .fill(btn_fill)
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0));

            if ui.add_enabled(is_ready, btn).clicked() {
                app.export_gcode();
            }
        });
    }

    fn export_gcode(&mut self) {
        let lang = self.config.language;
        if self.vector_doc.is_empty() {
            let msg = match lang {
                Language::Russian => "Нет элементов для экспорта! Добавьте текст или загрузите SVG.",
                Language::English => "No elements to export! Add text or load SVG.",
            };
            self.status_message = msg.to_string();
        } else if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-Code", &["gcode"])
            .set_file_name("plot.gcode")
            .save_file()
        {
            let raw_polylines = self.vector_doc.get_transformed_polylines();
            let polylines = if self.optimize_paths {
                PathOptimizer::optimize(raw_polylines)
            } else {
                raw_polylines
            };
            let gcode_text = crate::gcode::generator::GcodeGenerator::generate_with_multipass(
                &polylines,
                &self.config,
                self.multipass_count,
            );

            let mut out_of_bounds = false;
            for poly in &polylines {
                for pt in &poly.points {
                    let nx = pt.x - self.config.toolhead_offset_x;
                    let ny = pt.y - self.config.toolhead_offset_y;
                    let nozzle_oob = nx < -0.1 || nx > self.config.bed_width + 0.1 || ny < -0.1 || ny > self.config.bed_height + 0.1;
                    let pen_oob = pt.x < -0.1 || pt.x > self.config.bed_width + 0.1 || pt.y < -0.1 || pt.y > self.config.bed_height + 0.1;
                    if nozzle_oob || pen_oob {
                        out_of_bounds = true;
                        break;
                    }
                }
                if out_of_bounds {
                    break;
                }
            }

            if let Err(e) = std::fs::write(&path, gcode_text) {
                self.status_message = format!("Error: {}", e);
            } else if out_of_bounds {
                let warn = match lang {
                    Language::Russian => "⚠️ Файл сохранен, но обнаружен выход каретки за пределы станка!",
                    Language::English => "⚠️ Saved, but toolhead moves exceed machine limits!",
                };
                self.status_message = warn.to_string();
            } else {
                let ok = match lang {
                    Language::Russian => format!("Успешно сохранен: {:?}", path.file_name().unwrap_or_default()),
                    Language::English => format!("Successfully exported: {:?}", path.file_name().unwrap_or_default()),
                };
                self.status_message = ok;
            }
        }
    }

    /// Strict Card: background #18181b, border 1px #3f3f46, padding 12px, header 11px uppercase #a1a1aa
    fn render_card<F>(&mut self, ui: &mut Ui, title: &str, content: F)
    where
        F: FnOnce(&mut Ui, &mut Self),
    {
        egui::Frame::none()
            .fill(Color32::from_rgb(24, 24, 27))                          // #18181b
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))   // 1px #3f3f46
            .rounding(Rounding::same(6.0))                                 // 6px
            .inner_margin(12.0)                                            // padding 12px
            .show(ui, |ui| {
                ui.label(
                    RichText::new(title)
                        .font(FontId::new(11.0, FontFamily::Name("Inter-SemiBold".into())))
                        .color(Color32::from_rgb(161, 161, 170)),          // #a1a1aa
                );
                ui.add_space(6.0);
                content(ui, self);
            });
    }

    /// Setup Wizard Modal: background #18181b, border 1px #3f3f46, 6px rounding
    fn render_setup_wizard_modal(&mut self, ctx: &Context) {
        if !self.show_setup_wizard {
            return;
        }

        let lang = self.config.language;
        let mm_suf = format!(" {}", Tr::unit_mm(lang));

        egui::Window::new(Tr::wizard_title(lang))
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(18.0)
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(160),
                    }),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(Tr::wizard_title(lang))
                        .font(FontId::new(15.0, FontFamily::Name("Inter-SemiBold".into())))
                        .color(Color32::from_rgb(244, 244, 245)),
                );
                ui.label(
                    RichText::new(Tr::wizard_subtitle(lang))
                        .small()
                        .color(Color32::from_rgb(161, 161, 170)),
                );
                ui.add_space(8.0);

                ui.label(RichText::new(Tr::printer_model_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                let presets = PrinterProfile::presets();
                egui::Grid::new("wiz_grid").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
                    for (i, preset) in presets.iter().enumerate() {
                        let is_sel = self.config.name == preset.name;
                        let card_fill = if is_sel {
                            Color32::from_rgb(39, 39, 42) // #27272a
                        } else {
                            Color32::from_rgb(24, 24, 27) // #18181b
                        };
                        let stroke = if is_sel {
                            Stroke::new(1.0_f32, Color32::from_rgb(59, 130, 246)) // #3b82f6
                        } else {
                            Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70))  // #3f3f46
                        };

                        egui::Frame::none()
                            .fill(card_fill)
                            .stroke(stroke)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                if ui.selectable_label(is_sel, RichText::new(&preset.name).color(Color32::from_rgb(244, 244, 245))).clicked() {
                                    let current_lang = self.config.language;
                                    self.config = preset.clone();
                                    self.config.language = current_lang;
                                }
                                ui.label(
                                    RichText::new(format!("{:.0}×{:.0} {} | ΔX: {:.1}, ΔY: {:.1}", preset.bed_width, preset.bed_height, Tr::unit_mm(lang), preset.toolhead_offset_x, preset.toolhead_offset_y))
                                        .small()
                                        .color(Color32::from_rgb(161, 161, 170)),
                                );
                            });

                        if (i + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                    if presets.len() % 2 != 0 {
                        ui.end_row();
                    }
                });

                ui.add_space(10.0);
                ui.label(RichText::new(Tr::bed_and_offset_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                egui::Grid::new("wiz_inputs").num_columns(4).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label(RichText::new(Tr::bed_x_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.add(egui::DragValue::new(&mut self.config.bed_width).speed(1.0).range(50.0..=1000.0).suffix(&mm_suf));
                    ui.label(RichText::new(Tr::bed_y_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.add(egui::DragValue::new(&mut self.config.bed_height).speed(1.0).range(50.0..=1000.0).suffix(&mm_suf));
                    ui.end_row();

                    ui.label(RichText::new(Tr::offset_dx_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.add(egui::DragValue::new(&mut self.config.toolhead_offset_x).speed(0.5).range(-200.0..=200.0).suffix(&mm_suf));
                    ui.label(RichText::new(Tr::offset_dy_label(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.add(egui::DragValue::new(&mut self.config.toolhead_offset_y).speed(0.5).range(-200.0..=200.0).suffix(&mm_suf));
                    ui.end_row();
                });

                ui.add_space(12.0);
                let btn = egui::Button::new(
                    RichText::new(Tr::save_settings_btn(lang))
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(59, 130, 246)) // #3b82f6
                .stroke(Stroke::NONE)
                .rounding(Rounding::same(6.0));

                if ui.add_sized([ui.available_width(), 32.0], btn).clicked() {
                    self.config.is_first_run = false;
                    let _ = self.config.save();
                    self.vector_doc.center_on_safe_area(&self.config);
                    self.show_setup_wizard = false;
                    self.status_message = match lang {
                        Language::Russian => format!("Конфигурация сохранена: {}", self.config.name),
                        Language::English => format!("Configuration saved: {}", self.config.name),
                    };
                }
            });
    }

    fn render_about_dialog(&mut self, ctx: &Context) {
        if !self.show_about_dialog {
            return;
        }

        let lang = self.config.language;
        egui::Window::new(Tr::about_title(lang))
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .default_width(340.0)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(18.0)
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(160),
                    }),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("BedPlotter").font(FontId::new(15.0, FontFamily::Name("Inter-SemiBold".into()))).color(Color32::from_rgb(244, 244, 245)));
                let ver_str = match lang {
                    Language::Russian => "Версия 0.6.5 (Studio UI)",
                    Language::English => "Version 0.6.5 (Studio UI)",
                };
                ui.label(RichText::new(ver_str).small().color(Color32::from_rgb(161, 161, 170)));
                ui.separator();
                ui.label(Tr::about_desc(lang));
                ui.add_space(8.0);
                if ui.button(Tr::close_btn(lang)).clicked() {
                    self.show_about_dialog = false;
                }
            });
    }

    fn render_shortcuts_dialog(&mut self, ctx: &Context) {
        if !self.show_shortcuts_dialog {
            return;
        }

        let lang = self.config.language;
        egui::Window::new(Tr::shortcuts_title(lang))
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(18.0)
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(160),
                    }),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new(Tr::mouse_controls_title(lang)).font(FontId::new(13.0, FontFamily::Name("Inter-SemiBold".into()))).color(Color32::from_rgb(244, 244, 245)));
                let (m1, m2, m3, m4, m5) = match lang {
                    Language::Russian => (
                        "• Клик по столу: прямое создание фигуры (Esc для отмены)",
                        "• ЛКМ по углам рамки: масштабирование (Drag-to-Resize)",
                        "• ЛКМ по фигуре: перемещение по столу",
                        "• ПКМ / СКМ: панорамирование камеры",
                        "• Колесико мыши: плавный зум",
                    ),
                    Language::English => (
                        "• Click on bed: direct shape placement (Esc to cancel)",
                        "• LMB on box corners: resize (Drag-to-Resize)",
                        "• LMB on shape: move on bed",
                        "• RMB / MMB: pan camera",
                        "• Mouse wheel: smooth zoom",
                    ),
                };
                ui.label(m1);
                ui.label(m2);
                ui.label(m3);
                ui.label(m4);
                ui.label(m5);

                ui.add_space(6.0);
                ui.label(RichText::new(Tr::shortcuts_list_title(lang)).font(FontId::new(13.0, FontFamily::Name("Inter-SemiBold".into()))).color(Color32::from_rgb(244, 244, 245)));
                let (k1, k2, k3) = match lang {
                    Language::Russian => (
                        "• Cmd+O: импорт файла SVG",
                        "• Cmd+S: экспорт файла .gcode",
                        "• Esc: сброс текущего инструмента в Выделение",
                    ),
                    Language::English => (
                        "• Cmd+O: import SVG file",
                        "• Cmd+S: export .gcode file",
                        "• Esc: reset tool to Select",
                    ),
                };
                ui.label(k1);
                ui.label(k2);
                ui.label(k3);
                ui.add_space(8.0);
                if ui.button(Tr::close_btn(lang)).clicked() {
                    self.show_shortcuts_dialog = false;
                }
            });
    }
}

impl eframe::App for BedPlotterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.config.language;
        // Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.active_tool = ToolId::Select;
            self.status_message = match lang {
                Language::Russian => "Инструмент: Выделение".to_string(),
                Language::English => "Tool: Select".to_string(),
            };
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::O)) {
            if let Some(path) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).pick_file() {
                let _ = self.vector_doc.load_svg(&path);
                self.vector_doc.center_on_safe_area(&self.config);
                self.status_message = match lang {
                    Language::Russian => format!("Загружен SVG: {}", self.vector_doc.name),
                    Language::English => format!("Loaded SVG: {}", self.vector_doc.name),
                };
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            self.export_gcode();
        }

        // Animation update for G-code playback
        if self.preview.is_playing {
            self.preview.progress += 0.005;
            if self.preview.progress >= 1.0 {
                self.preview.progress = 1.0;
                self.preview.is_playing = false;
            }
            ctx.request_repaint();
        }

        // 1. OrcaSlicer Studio Header (44px): rigid, #18181b, border-bottom #3f3f46
        TopBottomPanel::top("unified_header")
            .exact_height(44.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 27))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::ZERO)
                    .inner_margin(egui::Margin::symmetric(12.0, 6.0)),
            )
            .show(ctx, |ui| {
                self.render_unified_header(ui, ctx);
            });

        // 2. Status Bar (24px): rigid, #18181b, border-top #3f3f46
        TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 27))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::ZERO)
                    .inner_margin(egui::Margin::symmetric(12.0, 3.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_message).small().color(Color32::from_rgb(161, 161, 170)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(pos) = self.cursor_bed_pos {
                            ui.label(
                                RichText::new(Tr::status_coords(lang, pos.x, pos.y))
                                    .small()
                                    .color(Color32::from_rgb(244, 244, 245)),
                            );
                        } else {
                            ui.label(RichText::new(Tr::outside_bed(lang)).small().color(Color32::from_rgb(161, 161, 170)));
                        }
                        ui.label(RichText::new(format!("{} {:.1}x", Tr::zoom_label(lang), self.viewport.zoom)).small().color(Color32::from_rgb(161, 161, 170)));
                    });
                });
            });

        // 3. Left Sidebar (44px): rigid, #18181b, border-right #3f3f46
        SidePanel::left("left_sidebar")
            .exact_width(44.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 27))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::ZERO),
            )
            .show(ctx, |ui| {
                self.render_left_sidebar(ui, ctx);
            });

        // 4. Right Sidebar (270px): rigid, #18181b, border-left #3f3f46, padding 12px
        SidePanel::right("right_sidebar")
            .default_width(270.0)
            .width_range(260.0..=360.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 27))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                    .rounding(Rounding::ZERO)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                self.render_right_sidebar(ui, ctx);
            });

        // 5. Central Canvas: matte background #121214
        CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(18, 18, 20)))
            .show(ctx, |ui| {
                let (canvas_rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                match self.mode {
                    AppMode::Calibration => {
                        self.calibration.render_calibration_editor(ui, canvas_rect);
                    }
                    AppMode::BedView => {
                        // Handle Pan & Zoom
                        self.viewport.handle_input(ui, &response);

                        // Cursor coordinates in mm
                        let mut hovered_bed_pos: Option<egui::Pos2> = None;
                        let hover_screen_pt = response.hover_pos();
                        if let Some(hover_pos) = hover_screen_pt {
                            let bed_pos = self.viewport.screen_to_bed(hover_pos, canvas_rect, &self.config);
                            if bed_pos.x >= 0.0
                                && bed_pos.x <= self.config.bed_width
                                && bed_pos.y >= 0.0
                                && bed_pos.y <= self.config.bed_height
                            {
                                self.cursor_bed_pos = Some(bed_pos);
                                hovered_bed_pos = Some(bed_pos);
                            } else {
                                self.cursor_bed_pos = None;
                            }
                        } else {
                            self.cursor_bed_pos = None;
                        }

                        // Cursor Icon Management & Corner Handle Detection
                        let mut corner_hovered: Option<usize> = None;
                        if self.active_tool == ToolId::Select && !self.vector_doc.is_empty() {
                            if let Some(corners) = self.viewport.get_corner_handles(canvas_rect, &self.config, &self.vector_doc) {
                                if let Some(m_pos) = hover_screen_pt {
                                    for (i, c) in corners.iter().enumerate() {
                                        if m_pos.distance(*c) <= 9.0 {
                                            corner_hovered = Some(i);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if corner_hovered.is_some() || self.vector_doc.is_resizing {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNorthWest);
                        } else if self.active_tool != ToolId::Select && self.active_tab != StepTab::Calibrate {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                        }

                        // Handle Interactive Corner Resize vs Direct Shape Placement vs Move Drag
                        if response.drag_started_by(egui::PointerButton::Primary) {
                            if let Some(c_idx) = corner_hovered {
                                self.vector_doc.is_resizing = true;
                                self.active_resize_corner = Some(c_idx);
                            } else if self.active_tool == ToolId::Select && !self.vector_doc.is_empty() {
                                if let Some(bed_pos) = hovered_bed_pos {
                                    if let Some(hit_idx) = self.vector_doc.item_at_bed_pos(bed_pos) {
                                        self.vector_doc.select_item(hit_idx);
                                        self.sync_text_controls_from_selection();
                                        self.vector_doc.is_dragging = true;
                                    } else if let Some(bb) = self.vector_doc.bounding_box() {
                                        if bb.expand(8.0).contains(bed_pos) {
                                            self.vector_doc.is_dragging = true;
                                        }
                                    }
                                }
                            }
                        }

                        // Direct Canvas Click-to-Place Tool Action (Multi-shape composition without erasing!)
                        if response.clicked_by(egui::PointerButton::Primary) && self.active_tab == StepTab::Prepare {
                            if let Some(pt) = hovered_bed_pos {
                                match self.active_tool {
                                    ToolId::Select => {
                                        if let Some(hit_idx) = self.vector_doc.item_at_bed_pos(pt) {
                                            self.vector_doc.select_item(hit_idx);
                                            self.sync_text_controls_from_selection();
                                        }
                                    }
                                    ToolId::Rectangle => {
                                        self.vector_doc.add_rectangle_at(self.rect_w, self.rect_h, pt, self.hatch_enabled, self.hatch_angle, self.hatch_spacing, self.hatch_cross);
                                        self.status_message = match lang {
                                            Language::Russian => format!("Прямоугольник размещен в ({:.1}, {:.1}) мм", pt.x, pt.y),
                                            Language::English => format!("Rectangle placed at ({:.1}, {:.1}) mm", pt.x, pt.y),
                                        };
                                    }
                                    ToolId::Circle => {
                                        self.vector_doc.add_circle_at(self.circle_r, pt, 48, self.hatch_enabled, self.hatch_angle, self.hatch_spacing, self.hatch_cross);
                                        self.status_message = match lang {
                                            Language::Russian => format!("Круг размещен в ({:.1}, {:.1}) мм", pt.x, pt.y),
                                            Language::English => format!("Circle placed at ({:.1}, {:.1}) mm", pt.x, pt.y),
                                        };
                                    }
                                    ToolId::Star => {
                                        self.vector_doc.add_star_at(self.star_r_out, self.star_r_in, self.star_pts, pt, self.hatch_enabled, self.hatch_angle, self.hatch_spacing, self.hatch_cross);
                                        self.status_message = match lang {
                                            Language::Russian => format!("Звезда размещена в ({:.1}, {:.1}) мм", pt.x, pt.y),
                                            Language::English => format!("Star placed at ({:.1}, {:.1}) mm", pt.x, pt.y),
                                        };
                                    }
                                    ToolId::Text => {
                                        let h_str = if self.humanizer_enabled { self.humanizer_strength } else { 0.0 };
                                        if let Some(hit_idx) = self.vector_doc.item_at_bed_pos(pt) {
                                            self.vector_doc.select_item(hit_idx);
                                            self.sync_text_controls_from_selection();
                                        } else {
                                            self.vector_doc.add_text_at(&self.text_input, self.text_size_mm, pt, self.font_style, h_str);
                                            self.status_message = match lang {
                                                Language::Russian => format!("Текст размещен в ({:.1}, {:.1}) мм", pt.x, pt.y),
                                                Language::English => format!("Text placed at ({:.1}, {:.1}) mm", pt.x, pt.y),
                                            };
                                        }
                                    }
                                    ToolId::TestGrid => {
                                        self.vector_doc.add_test_pattern_at(self.test_size, self.test_step, pt);
                                        self.status_message = match lang {
                                            Language::Russian => format!("Сетка размещена в ({:.1}, {:.1}) мм", pt.x, pt.y),
                                            Language::English => format!("Calibration grid placed at ({:.1}, {:.1}) mm", pt.x, pt.y),
                                        };
                                    }
                                }
                            }
                        }

                        // Hotkey: Delete active selected item (ONLY when not typing numbers/text!)
                        if !ui.ctx().wants_keyboard_input() && !ui.ctx().memory(|m| m.focused().is_some()) {
                            if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                                if self.active_tool == ToolId::Select {
                                    if let Some(idx) = self.vector_doc.selected_index {
                                        self.vector_doc.remove_item(idx);
                                    }
                                }
                            }
                        }

                        // Drag logic: Resizing
                        if response.dragged_by(egui::PointerButton::Primary) && self.vector_doc.is_resizing {
                            let drag_delta = response.drag_delta();
                            let center_screen = self.viewport.bed_to_screen(self.vector_doc.position, canvas_rect, &self.config);
                            if let Some(m_pos) = hover_screen_pt {
                                let cur_dist = (m_pos - center_screen).length();
                                let prev_dist = ((m_pos - drag_delta) - center_screen).length();
                                if prev_dist > 5.0 && cur_dist > 5.0 {
                                    let factor = cur_dist / prev_dist;
                                    self.vector_doc.scale = (self.vector_doc.scale * factor).clamp(0.05, 10.0);
                                    self.vector_doc.sync_to_selected();
                                }
                            }
                        }

                        // Drag logic: Moving
                        if response.dragged_by(egui::PointerButton::Primary) && self.vector_doc.is_dragging {
                            let drag_delta = response.drag_delta();
                            let delta_x_mm = drag_delta.x / self.viewport.zoom;
                            let delta_y_mm = -drag_delta.y / self.viewport.zoom;

                            self.vector_doc.position.x += delta_x_mm;
                            self.vector_doc.position.y += delta_y_mm;

                            self.vector_doc.position.x = self
                                .vector_doc
                                .position
                                .x
                                .clamp(-50.0, self.config.bed_width + 50.0);
                            self.vector_doc.position.y = self
                                .vector_doc
                                .position
                                .y
                                .clamp(-50.0, self.config.bed_height + 50.0);
                            self.vector_doc.sync_to_selected();
                        }

                        if response.drag_stopped_by(egui::PointerButton::Primary) {
                            self.vector_doc.is_dragging = false;
                            self.vector_doc.is_resizing = false;
                            self.active_resize_corner = None;
                        }

                        let painter = ui.painter_at(canvas_rect);

                        // 1. Render Bed Base & Precision Millimeter Grid
                        self.viewport.render_bed_and_grid(&painter, canvas_rect, &self.config);

                        // 2. Render Rectified Photo on Bed (if calibrated)
                        if let Some(rectified_tex) = &self.calibration.rectified_texture {
                            self.viewport.render_rectified_photo(
                                &painter,
                                canvas_rect,
                                &self.config,
                                rectified_tex,
                                self.calibration.photo_opacity,
                            );
                        }

                        // 3. Render Ghost Preview when placing shapes (only in Prepare mode)
                        if self.active_tool != ToolId::Select && self.active_tab == StepTab::Prepare {
                            if let Some(bed_pt) = hovered_bed_pos {
                                match self.active_tool {
                                    ToolId::Rectangle => {
                                        self.viewport.render_ghost_preview(&painter, canvas_rect, &self.config, bed_pt, self.rect_w * 0.5, self.rect_h * 0.5, false);
                                    }
                                    ToolId::Circle => {
                                        self.viewport.render_ghost_preview(&painter, canvas_rect, &self.config, bed_pt, self.circle_r, self.circle_r, true);
                                    }
                                    ToolId::Star => {
                                        self.viewport.render_ghost_preview(&painter, canvas_rect, &self.config, bed_pt, self.star_r_out, self.star_r_out, true);
                                    }
                                    ToolId::Text => {
                                        let est_w = self.text_input.len() as f32 * self.text_size_mm * 0.7;
                                        self.viewport.render_ghost_preview(&painter, canvas_rect, &self.config, bed_pt, est_w * 0.5, self.text_size_mm * 0.5, false);
                                    }
                                    ToolId::TestGrid => {
                                        self.viewport.render_ghost_preview(&painter, canvas_rect, &self.config, bed_pt, self.test_size * 0.5, self.test_size * 0.5, false);
                                    }
                                    ToolId::Select => {}
                                }
                            }
                        }

                        // 4. Render Vector Document (polylines, text, shapes)
                        let (preview_prog, override_paths) = if self.active_tab == StepTab::Preview {
                            self.update_opt_cache_if_needed();
                            let paths = if self.optimize_paths {
                                self.cached_opt_paths.clone()
                            } else {
                                None
                            };
                            (Some(self.preview.progress), paths)
                        } else {
                            (None, None)
                        };

                        self.viewport.render_vector(
                            &painter,
                            canvas_rect,
                            &self.config,
                            &self.vector_doc,
                            preview_prog,
                            override_paths,
                        );

                        // 5. Viewport HUD Controls (Bottom-right of canvas - prevents dropdown collision)
                        let hud_w = 176.0;
                        let hud_h = 32.0;
                        let hud_rect = Rect::from_min_size(
                            pos2(canvas_rect.right() - hud_w - 12.0, canvas_rect.bottom() - hud_h - 12.0),
                            vec2(hud_w, hud_h),
                        );

                        painter.rect_filled(
                            hud_rect,
                            6.0,
                            Color32::from_rgb(24, 24, 27), // #18181b
                        );
                        painter.rect_stroke(
                            hud_rect,
                            6.0,
                            Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)), // #3f3f46
                        );

                        let mut hx = hud_rect.min.x + 4.0;
                        let hy = hud_rect.min.y + 3.0;

                        let style_hud_btn = |text: &str| {
                            egui::Button::new(
                                RichText::new(text)
                                    .small()
                                    .color(Color32::from_rgb(244, 244, 245)),
                            )
                            .fill(Color32::from_rgb(39, 39, 42))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)))
                            .rounding(Rounding::same(6.0))
                        };

                        if ui.put(Rect::from_min_size(pos2(hx, hy), vec2(26.0, 26.0)), style_hud_btn("-")).clicked() {
                            self.viewport.zoom = (self.viewport.zoom / 1.2).clamp(0.4, 25.0);
                        }
                        hx += 30.0;

                        // 1:1 Physical Scale
                        if ui.put(Rect::from_min_size(pos2(hx, hy), vec2(46.0, 26.0)), style_hud_btn("1:1")).clicked() {
                            self.viewport.zoom = 2.5;
                            self.viewport.pan = egui::Vec2::ZERO;
                        }
                        hx += 50.0;

                        if ui.put(Rect::from_min_size(pos2(hx, hy), vec2(26.0, 26.0)), style_hud_btn("+")).clicked() {
                            self.viewport.zoom = (self.viewport.zoom * 1.2).clamp(0.4, 25.0);
                        }
                        hx += 30.0;

                        // Fit whole bed
                        let bed_btn_label = if self.config.language == Language::Russian { "Стол" } else { "Bed" };
                        if ui.put(Rect::from_min_size(pos2(hx, hy), vec2(56.0, 26.0)), style_hud_btn(bed_btn_label)).clicked() {
                            self.viewport.reset_view(canvas_rect, &self.config);
                        }
                    }
                }
            });

        // 6. Full-screen Modal Backdrop Scrim (Order::Middle dims background panels, while modal windows on Order::Foreground remain 100% bright and elevated)
        if self.show_setup_wizard || self.show_about_dialog || self.show_shortcuts_dialog {
            let screen_rect = ctx.screen_rect();
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("modal_scrim_layer")))
                .rect_filled(screen_rect, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 90));
        }

        // 7. Modals
        self.render_setup_wizard_modal(ctx);
        self.render_about_dialog(ctx);
        self.render_shortcuts_dialog(ctx);
    }
}
