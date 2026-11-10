use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Russian,
    English,
}

#[allow(dead_code)]
impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::Russian => "Русский",
            Language::English => "English",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Russian => "RU",
            Language::English => "EN",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Russian => Language::English,
            Language::English => Language::Russian,
        }
    }
}

pub struct Tr;

#[allow(dead_code)]
impl Tr {
    pub fn menu(l: Language) -> &'static str {
        match l {
            Language::Russian => "Меню ▾",
            Language::English => "Menu ▾",
        }
    }

    pub fn language(l: Language) -> &'static str {
        match l {
            Language::Russian => "Язык / Language",
            Language::English => "Language / Язык",
        }
    }

    pub fn import_svg(l: Language) -> &'static str {
        match l {
            Language::Russian => "Импорт SVG... (Cmd+O)",
            Language::English => "Import SVG... (Cmd+O)",
        }
    }

    pub fn load_bed_photo(l: Language) -> &'static str {
        match l {
            Language::Russian => "Загрузить фото стола...",
            Language::English => "Load Bed Photo...",
        }
    }

    pub fn printer_wizard(l: Language) -> &'static str {
        match l {
            Language::Russian => "Мастер станка (Профили)...",
            Language::English => "Printer Wizard (Profiles)...",
        }
    }

    pub fn about(l: Language) -> &'static str {
        match l {
            Language::Russian => "О программе BedPlotter",
            Language::English => "About BedPlotter",
        }
    }

    pub fn shortcuts(l: Language) -> &'static str {
        match l {
            Language::Russian => "Горячие клавиши...",
            Language::English => "Keyboard Shortcuts...",
        }
    }

    pub fn tab_prepare(l: Language) -> &'static str {
        match l {
            Language::Russian => "Подготовка",
            Language::English => "Prepare",
        }
    }

    pub fn tab_calibrate(l: Language) -> &'static str {
        match l {
            Language::Russian => "Калибровка",
            Language::English => "Calibrate",
        }
    }

    pub fn tab_preview(l: Language) -> &'static str {
        match l {
            Language::Russian => "Предпросмотр",
            Language::English => "Preview",
        }
    }

    pub fn export_gcode(l: Language) -> &'static str {
        match l {
            Language::Russian => "Экспорт G-code",
            Language::English => "Export G-code",
        }
    }

    pub fn configure_printers(l: Language) -> &'static str {
        match l {
            Language::Russian => "⚙ Настроить станки...",
            Language::English => "⚙ Configure Printers...",
        }
    }

    pub fn card_creation(l: Language) -> &'static str {
        match l {
            Language::Russian => "СОЗДАНИЕ ОБЪЕКТА",
            Language::English => "OBJECT CREATION",
        }
    }

    pub fn card_transform(l: Language) -> &'static str {
        match l {
            Language::Russian => "ТРАНСФОРМАЦИЯ И ПОЛОЖЕНИЕ",
            Language::English => "TRANSFORM & POSITION",
        }
    }

    pub fn card_plotter(l: Language) -> &'static str {
        match l {
            Language::Russian => "ПАРАМЕТРЫ ПЛОТТЕРА",
            Language::English => "PLOTTER PARAMETERS",
        }
    }

    pub fn card_simulation(l: Language) -> &'static str {
        match l {
            Language::Russian => "СИМУЛЯЦИЯ ПЕЧАТИ",
            Language::English => "PRINT SIMULATION",
        }
    }

    pub fn card_stats(l: Language) -> &'static str {
        match l {
            Language::Russian => "СТАТИСТИКА И СКОРОСТЬ",
            Language::English => "STATISTICS & SPEED",
        }
    }

    pub fn tool_select(l: Language) -> &'static str {
        match l {
            Language::Russian => "Выделение и трансформация (V)",
            Language::English => "Select & Transform (V)",
        }
    }

    pub fn tool_text(l: Language) -> &'static str {
        match l {
            Language::Russian => "Однолинейный векторный текст (T)",
            Language::English => "Single-line Vector Text (T)",
        }
    }

    pub fn tool_rect(l: Language) -> &'static str {
        match l {
            Language::Russian => "Прямоугольник (R)",
            Language::English => "Rectangle (R)",
        }
    }

    pub fn tool_circle(l: Language) -> &'static str {
        match l {
            Language::Russian => "Окружность / Круг (C)",
            Language::English => "Circle (C)",
        }
    }

    pub fn tool_star(l: Language) -> &'static str {
        match l {
            Language::Russian => "Многолучевая звезда (S)",
            Language::English => "Star (S)",
        }
    }

    pub fn tool_grid(l: Language) -> &'static str {
        match l {
            Language::Russian => "Калибровочная сетка стола (G)",
            Language::English => "Bed Calibration Grid (G)",
        }
    }

    pub fn font_style(l: Language) -> &'static str {
        match l {
            Language::Russian => "Стиль шрифта:",
            Language::English => "Font Style:",
        }
    }

    pub fn font_gost(l: Language) -> &'static str {
        match l {
            Language::Russian => "Чертежный (ГОСТ)",
            Language::English => "Technical (ISO)",
        }
    }

    pub fn font_cursive(l: Language) -> &'static str {
        match l {
            Language::Russian => "Рукописный (Cursive)",
            Language::English => "Handwritten (Cursive)",
        }
    }

    pub fn text_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Текст:",
            Language::English => "Text:",
        }
    }

    pub fn height_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Высота:",
            Language::English => "Height:",
        }
    }

    pub fn add_to_center(l: Language) -> &'static str {
        match l {
            Language::Russian => "Добавить по центру стола",
            Language::English => "Add to Bed Center",
        }
    }

    pub fn humanizer_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Эффект живой руки (Humanizer)",
            Language::English => "Hand-drawn Humanizer",
        }
    }

    pub fn humanizer_strength(l: Language) -> &'static str {
        match l {
            Language::Russian => "Сила колебаний:",
            Language::English => "Jitter Strength:",
        }
    }

    pub fn hatch_infill(l: Language) -> &'static str {
        match l {
            Language::Russian => "Штриховка / Заливка (Hatch)",
            Language::English => "Hatch Infill / Shading",
        }
    }

    pub fn hatch_angle(l: Language) -> &'static str {
        match l {
            Language::Russian => "Угол штриховки:",
            Language::English => "Hatch Angle:",
        }
    }

    pub fn hatch_spacing(l: Language) -> &'static str {
        match l {
            Language::Russian => "Шаг линий:",
            Language::English => "Line Spacing:",
        }
    }

    pub fn cross_hatch(l: Language) -> &'static str {
        match l {
            Language::Russian => "Перекрестная штриховка (Cross)",
            Language::English => "Cross-hatch (2 directions)",
        }
    }

    pub fn multipass(l: Language) -> &'static str {
        match l {
            Language::Russian => "Повторные проходы (Multipass):",
            Language::English => "Pass count (Multipass):",
        }
    }

    pub fn ready_status(l: Language) -> &'static str {
        match l {
            Language::Russian => "Готов к работе.",
            Language::English => "Ready.",
        }
    }

    pub fn optimize_paths(l: Language) -> &'static str {
        match l {
            Language::Russian => "Оптимизация холостых ходов (TSP)",
            Language::English => "Optimize Travel Moves (TSP)",
        }
    }

    pub fn width(l: Language) -> &'static str {
        match l {
            Language::Russian => "Ширина:",
            Language::English => "Width:",
        }
    }

    pub fn length(l: Language) -> &'static str {
        match l {
            Language::Russian => "Длина:",
            Language::English => "Length:",
        }
    }

    pub fn radius(l: Language) -> &'static str {
        match l {
            Language::Russian => "Радиус:",
            Language::English => "Radius:",
        }
    }

    pub fn rays(l: Language) -> &'static str {
        match l {
            Language::Russian => "Лучей:",
            Language::English => "Points:",
        }
    }

    pub fn outer_radius(l: Language) -> &'static str {
        match l {
            Language::Russian => "R внеш:",
            Language::English => "Outer R:",
        }
    }

    pub fn inner_radius(l: Language) -> &'static str {
        match l {
            Language::Russian => "R внутр:",
            Language::English => "Inner R:",
        }
    }

    pub fn fit_to_bed(l: Language) -> &'static str {
        match l {
            Language::Russian => "Вписать в стол (с отступом 10мм)",
            Language::English => "Fit to Bed (10mm margin)",
        }
    }

    pub fn center_on_bed(l: Language) -> &'static str {
        match l {
            Language::Russian => "По центру стола",
            Language::English => "Center on Bed",
        }
    }

    pub fn clear(l: Language) -> &'static str {
        match l {
            Language::Russian => "Очистить холст",
            Language::English => "Clear Canvas",
        }
    }

    // Units
    pub fn unit_mm(l: Language) -> &'static str {
        match l {
            Language::Russian => "мм",
            Language::English => "mm",
        }
    }

    pub fn unit_speed(l: Language) -> &'static str {
        match l {
            Language::Russian => "мм/мин",
            Language::English => "mm/min",
        }
    }

    // Calibration Card
    pub fn card_calibration(l: Language) -> &'static str {
        match l {
            Language::Russian => "КАЛИБРОВКА ПО СНИМКУ СТОЛА",
            Language::English => "BED PHOTO CALIBRATION",
        }
    }

    pub fn load_bed_photo_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "Загрузить фото стола...",
            Language::English => "Load Bed Photo...",
        }
    }

    pub fn calibration_target_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Цель калибровки:",
            Language::English => "Calibration Target:",
        }
    }

    pub fn target_full_bed(l: Language) -> &'static str {
        match l {
            Language::Russian => "Весь стол",
            Language::English => "Full Bed",
        }
    }

    pub fn target_a4(l: Language) -> &'static str {
        match l {
            Language::Russian => "Лист А4",
            Language::English => "A4 Sheet",
        }
    }

    pub fn reset_points(l: Language) -> &'static str {
        match l {
            Language::Russian => "Сбросить 4 точки",
            Language::English => "Reset 4 Points",
        }
    }

    pub fn rectify_perspective(l: Language) -> &'static str {
        match l {
            Language::Russian => "Выровнять перспективу (1:1)",
            Language::English => "Rectify Perspective (1:1)",
        }
    }

    pub fn opacity(l: Language) -> &'static str {
        match l {
            Language::Russian => "Прозрачность:",
            Language::English => "Opacity:",
        }
    }

    pub fn calibrate_hint(l: Language) -> &'static str {
        match l {
            Language::Russian => "Загрузите снимок стола для выравнивания перспективы 1:1 по сетке.",
            Language::English => "Load a bed photo to rectify perspective 1:1 on the bed grid.",
        }
    }

    // Workspace & Toolhead Offset
    pub fn card_workspace_offset(l: Language) -> &'static str {
        match l {
            Language::Russian => "РАБОЧАЯ ЗОНА И ОФФСЕТ ПЕРА",
            Language::English => "WORKSPACE & TOOLHEAD OFFSET",
        }
    }

    pub fn printable_area(l: Language, w: f32, h: f32) -> String {
        match l {
            Language::Russian => format!("Поле рисования: {:.0} × {:.0} мм", w, h),
            Language::English => format!("Printable area: {:.0} × {:.0} mm", w, h),
        }
    }

    pub fn offset_relative_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Смещение пера относительно сопла:",
            Language::English => "Toolhead offset (relative to nozzle):",
        }
    }

    pub fn offset_hint_x(l: Language, x: f32) -> &'static str {
        match l {
            Language::Russian => {
                if x < 0.0 {
                    "(левее сопла)"
                } else if x > 0.0 {
                    "(правее сопла)"
                } else {
                    "(по центру X)"
                }
            }
            Language::English => {
                if x < 0.0 {
                    "(left of nozzle)"
                } else if x > 0.0 {
                    "(right of nozzle)"
                } else {
                    "(centered X)"
                }
            }
        }
    }

    pub fn offset_hint_y(l: Language, y: f32) -> &'static str {
        match l {
            Language::Russian => {
                if y > 0.0 {
                    "(спереди сопла)"
                } else if y < 0.0 {
                    "(сзади сопла)"
                } else {
                    "(по центру Y)"
                }
            }
            Language::English => {
                if y > 0.0 {
                    "(front of nozzle)"
                } else if y < 0.0 {
                    "(behind nozzle)"
                } else {
                    "(centered Y)"
                }
            }
        }
    }

    pub fn lift_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Подъем:",
            Language::English => "Pen Lift:",
        }
    }

    pub fn z_axis_desc(l: Language, up: f32, down: f32) -> String {
        match l {
            Language::Russian => format!("Z-ось (Z_up: {:.1}, Z_down: {:.1})", up, down),
            Language::English => format!("Z-axis (Z_up: {:.1}, Z_down: {:.1})", up, down),
        }
    }

    pub fn servo_desc(l: Language, pin: u8) -> String {
        match l {
            Language::Russian => format!("Сервопривод (Pin {})", pin),
            Language::English => format!("Servo (Pin {})", pin),
        }
    }

    pub fn configure_printer_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "⚙ Полные параметры принтера...",
            Language::English => "⚙ Configure Printer Profile...",
        }
    }

    // Pen Tip & Line Thickness
    pub fn card_pen_tip(l: Language) -> &'static str {
        match l {
            Language::Russian => "ТОЛЩИНА ЛИНИИ (ПЕРО)",
            Language::English => "PEN TIP & LINE THICKNESS",
        }
    }

    pub fn pen_tip_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Диаметр наконечника пера:",
            Language::English => "Pen tip / marker diameter:",
        }
    }

    pub fn pen_presets(l: Language) -> [(&'static str, f32); 4] {
        match l {
            Language::Russian => [
                ("0.3 Линер", 0.3),
                ("0.5 Ручка", 0.5),
                ("1.0 Маркер", 1.0),
                ("2.0 Фломастер", 2.0),
            ],
            Language::English => [
                ("0.3 Fineliner", 0.3),
                ("0.5 Ballpoint", 0.5),
                ("1.0 Marker", 1.0),
                ("2.0 Felt-tip", 2.0),
            ],
        }
    }

    // Quick Templates
    pub fn card_quick_templates(l: Language) -> &'static str {
        match l {
            Language::Russian => "БЫСТРЫЕ ШАБЛОНЫ БУМАГИ",
            Language::English => "QUICK PAPER TEMPLATES",
        }
    }

    pub fn quick_templates_hint(l: Language) -> &'static str {
        match l {
            Language::Russian => "Разместить контур листа по центру:",
            Language::English => "Place paper sheet at center:",
        }
    }

    pub fn quick_template_items(l: Language) -> [(&'static str, f32, f32); 4] {
        match l {
            Language::Russian => [
                ("А5 (148×210)", 148.0, 210.0),
                ("Открытка (100×150)", 100.0, 150.0),
                ("Квадрат (80×80)", 80.0, 80.0),
                ("Линейка (150×20)", 150.0, 20.0),
            ],
            Language::English => [
                ("A5 (148×210)", 148.0, 210.0),
                ("Postcard (100×150)", 100.0, 150.0),
                ("Square (80×80)", 80.0, 80.0),
                ("Ruler (150×20)", 150.0, 20.0),
            ],
        }
    }

    // Preview
    pub fn est_time(l: Language, m: u32, s: u32) -> String {
        match l {
            Language::Russian => format!("⏱ Время: {} мин {} сек", m, s),
            Language::English => format!("⏱ Estimated time: {}m {}s", m, s),
        }
    }

    pub fn preview_stats(l: Language, draw: f32, segs: usize, travel: f32) -> String {
        let draw = draw.max(0.0);
        let travel = travel.max(0.0);
        match l {
            Language::Russian => format!(
                "Длина линий: {:.1} мм  |  Отрезков: {}  |  Холостой: {:.1} мм",
                draw, segs, travel
            ),
            Language::English => format!(
                "Draw distance: {:.1} mm  |  Segments: {}  |  Travel: {:.1} mm",
                draw, segs, travel
            ),
        }
    }

    pub fn savings_tsp(l: Language, pct: f32) -> String {
        match l {
            Language::Russian => format!("Экономия холостого хода (TSP): -{:.0}%", pct),
            Language::English => format!("Travel savings (TSP): -{:.0}%", pct),
        }
    }

    pub fn progress_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Прогресс:",
            Language::English => "Progress:",
        }
    }

    pub fn play_btn(l: Language, is_playing: bool) -> &'static str {
        match (l, is_playing) {
            (Language::Russian, true) => "⏸ Пауза",
            (Language::Russian, false) => "▶ Воспроизвести",
            (Language::English, true) => "⏸ Pause",
            (Language::English, false) => "▶ Play",
        }
    }

    pub fn feedrate_travel_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Холостой ход (G0):",
            Language::English => "Travel feedrate (G0):",
        }
    }

    pub fn feedrate_draw_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Рисование (G1):",
            Language::English => "Draw feedrate (G1):",
        }
    }

    pub fn toolhead_offset_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Оффсет пера:",
            Language::English => "Toolhead offset:",
        }
    }

    // Status bar & Canvas
    pub fn outside_bed(l: Language) -> &'static str {
        match l {
            Language::Russian => "Вне стола",
            Language::English => "Outside bed",
        }
    }

    pub fn zoom_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Зум:",
            Language::English => "Zoom:",
        }
    }

    pub fn bed_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "Стол",
            Language::English => "Bed",
        }
    }

    pub fn status_coords(l: Language, x: f32, y: f32) -> String {
        match l {
            Language::Russian => format!("X: {:.1} мм  |  Y: {:.1} мм", x, y),
            Language::English => format!("X: {:.1} mm  |  Y: {:.1} mm", x, y),
        }
    }

    // Creation Tools
    pub fn new_text_block_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "+ Новый блок текста",
            Language::English => "+ New Text Block",
        }
    }

    pub fn add_rect_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "+ Добавить прямоугольник",
            Language::English => "+ Add Rectangle",
        }
    }

    pub fn add_circle_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "+ Добавить круг",
            Language::English => "+ Add Circle",
        }
    }

    pub fn add_star_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "+ Добавить звезду",
            Language::English => "+ Add Star",
        }
    }

    pub fn add_grid_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "+ Добавить калибровочную сетку",
            Language::English => "+ Add Calibration Grid",
        }
    }

    pub fn step_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Шаг:",
            Language::English => "Step:",
        }
    }

    pub fn layers_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "СЛОИ И ОБЪЕКТЫ НА СТОЛЕ",
            Language::English => "LAYERS & OBJECTS ON BED",
        }
    }

    pub fn merge_all_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "🔗 Объединить все объекты (Merge)",
            Language::English => "🔗 Merge All Objects",
        }
    }

    pub fn delete_object_tip(l: Language) -> &'static str {
        match l {
            Language::Russian => "Удалить этот объект",
            Language::English => "Delete this object",
        }
    }

    pub fn merge_tip(l: Language) -> &'static str {
        match l {
            Language::Russian => "Склеить все объекты в единую неделимую траекторию",
            Language::English => "Merge all objects into a single continuous path",
        }
    }

    pub fn scale_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Шкала:",
            Language::English => "Scale:",
        }
    }

    pub fn angle_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Угол:",
            Language::English => "Angle:",
        }
    }

    pub fn lock_aspect_ratio_label(l: Language, locked: bool) -> &'static str {
        match (l, locked) {
            (Language::Russian, true) => "🔗 Пропорции: Вкл",
            (Language::Russian, false) => "🔓 Пропорции: Выкл",
            (Language::English, true) => "🔗 Aspect: Locked",
            (Language::English, false) => "🔓 Aspect: Free",
        }
    }

    // Setup Wizard Modal
    pub fn wizard_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "Параметры 3D-принтера",
            Language::English => "3D Printer Settings",
        }
    }

    pub fn wizard_subtitle(l: Language) -> &'static str {
        match l {
            Language::Russian => "Выберите модель или задайте габариты стола и смещение инструмента:",
            Language::English => "Select a printer preset or configure bed dimensions and toolhead offset:",
        }
    }

    pub fn printer_model_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Модель принтера:",
            Language::English => "Printer Model:",
        }
    }

    pub fn bed_and_offset_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Габариты стола и оффсет пера (мм):",
            Language::English => "Bed Dimensions & Toolhead Offset (mm):",
        }
    }

    pub fn bed_x_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Стол X:",
            Language::English => "Bed X:",
        }
    }

    pub fn bed_y_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Стол Y:",
            Language::English => "Bed Y:",
        }
    }

    pub fn offset_dx_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Оффсет ΔX:",
            Language::English => "Offset ΔX:",
        }
    }

    pub fn offset_dy_label(l: Language) -> &'static str {
        match l {
            Language::Russian => "Оффсет ΔY:",
            Language::English => "Offset ΔY:",
        }
    }

    pub fn save_settings_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "Сохранить параметры",
            Language::English => "Save Settings",
        }
    }

    pub fn close_btn(l: Language) -> &'static str {
        match l {
            Language::Russian => "Закрыть",
            Language::English => "Close",
        }
    }

    // Dialogs: About & Shortcuts
    pub fn about_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "О программе BedPlotter",
            Language::English => "About BedPlotter",
        }
    }

    pub fn about_desc(l: Language) -> &'static str {
        match l {
            Language::Russian => "Профессиональный инструмент для перьевого плоттинга на FDM 3D-принтерах.",
            Language::English => "Professional pen plotting software for FDM 3D printers.",
        }
    }

    pub fn shortcuts_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "Горячие клавиши",
            Language::English => "Keyboard Shortcuts",
        }
    }

    pub fn mouse_controls_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "Управление мышью:",
            Language::English => "Mouse Controls:",
        }
    }

    pub fn shortcuts_list_title(l: Language) -> &'static str {
        match l {
            Language::Russian => "Горячие клавиши:",
            Language::English => "Keyboard Shortcuts:",
        }
    }
}
