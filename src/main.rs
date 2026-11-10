mod app;
mod canvas;
mod config;
mod gcode;
mod i18n;
mod vector;

use app::BedPlotterApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BedPlotter")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "BedPlotter",
        native_options,
        Box::new(|cc| Ok(Box::new(BedPlotterApp::new(cc)))),
    )
}
