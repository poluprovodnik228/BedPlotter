use crate::i18n::Language;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PenMechanism {
    ZAxis {
        z_safe: f32,
        z_draw: f32,
        feedrate_z: f32,
    },
    KlipperMacros {
        macro_up: String,
        macro_down: String,
    },
    Servo {
        angle_up: u8,
        angle_down: u8,
        pin: u8,
        delay_ms: u32,
    },
}

impl Default for PenMechanism {
    fn default() -> Self {
        Self::ZAxis {
            z_safe: 3.0,
            z_draw: 0.2,
            feedrate_z: 600.0,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_pen_tip() -> f32 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterProfile {
    #[serde(default = "default_true")]
    pub is_first_run: bool,
    #[serde(default)]
    pub language: Language,
    pub name: String,
    pub bed_width: f32,            // X max (mm)
    pub bed_height: f32,           // Y max (mm)
    pub toolhead_offset_x: f32,    // Toolhead offset X (marker relative to nozzle, mm)
    pub toolhead_offset_y: f32,    // Toolhead offset Y (marker relative to nozzle, mm)
    #[serde(default = "default_pen_tip")]
    pub pen_tip_diameter_mm: f32,  // Physical pen / marker tip diameter in mm (default 0.5)
    pub feedrate_travel: f32,      // G0 speed (mm/min)
    pub feedrate_draw: f32,        // G1 speed (mm/min)
    pub pen_mechanism: PenMechanism,
    pub klipper_url: String,       // e.g. "http://192.168.1.100" or "http://klipper.local"
    pub start_gcode: String,
    pub end_gcode: String,
}

impl Default for PrinterProfile {
    fn default() -> Self {
        let mut p = Self::anycubic_kobra_1();
        p.is_first_run = true;
        p
    }
}

impl PrinterProfile {
    /// Pure printer model name without any trailing dimensions, parentheses, or dots.
    pub fn clean_name(&self) -> String {
        let name = self.name.trim();
        let name = if let Some(idx) = name.find('(') {
            name[..idx].trim()
        } else if let Some(idx) = name.find('·') {
            name[..idx].trim()
        } else if let Some(idx) = name.find('•') {
            name[..idx].trim()
        } else {
            name
        };
        name.to_string()
    }

    /// Strict canonical display format: "Anycubic Kobra 1 (222×222)"
    pub fn formatted_name(&self) -> String {
        format!("{} ({:.0}×{:.0})", self.clean_name(), self.bed_width, self.bed_height)
    }

    /// Safe printable bounds for the pen in bed coordinates: (min_x, max_x, min_y, max_y).
    /// Since X_nozzle = X_pen - toolhead_offset_x, and 0 <= X_nozzle <= bed_width:
    /// toolhead_offset_x <= X_pen <= bed_width + toolhead_offset_x.
    /// In addition, the pen must physically stay within the bed: 0 <= X_pen <= bed_width.
    pub fn safe_pen_bounds(&self) -> (f32, f32, f32, f32) {
        let min_x = self.toolhead_offset_x.max(0.0);
        let max_x = (self.bed_width + self.toolhead_offset_x).min(self.bed_width);
        let min_y = self.toolhead_offset_y.max(0.0);
        let max_y = (self.bed_height + self.toolhead_offset_y).min(self.bed_height);
        (min_x, max_x, min_y, max_y)
    }

    /// Center point of the safe printable area for the pen
    pub fn safe_pen_center(&self) -> (f32, f32) {
        let (min_x, max_x, min_y, max_y) = self.safe_pen_bounds();
        ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
    }

    /// Dimensions of the safe printable area (width, height)
    pub fn safe_pen_dimensions(&self) -> (f32, f32) {
        let (min_x, max_x, min_y, max_y) = self.safe_pen_bounds();
        ((max_x - min_x).max(0.0), (max_y - min_y).max(0.0))
    }

    pub fn anycubic_kobra_1() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Anycubic Kobra 1".to_string(),
            bed_width: 222.0,
            bed_height: 222.0,
            toolhead_offset_x: -32.0,
            toolhead_offset_y: 12.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 3600.0,
            feedrate_draw: 1200.0,
            pen_mechanism: PenMechanism::ZAxis {
                z_safe: 2.5,
                z_draw: 0.2,
                feedrate_z: 600.0,
            },
            klipper_url: "http://klipper.local".to_string(),
            start_gcode: "G21 ; mm\nG90 ; absolute coordinates\nG28 ; home all\n".to_string(),
            end_gcode: "M400 ; wait for moves\nG0 Z10 F1000 ; lift pen\nG28 X Y ; park carriage\nM84 ; disable motors\n".to_string(),
        }
    }

    pub fn anycubic_kobra_2() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Anycubic Kobra 2 / Neo / Go".to_string(),
            bed_width: 220.0,
            bed_height: 220.0,
            toolhead_offset_x: -32.0,
            toolhead_offset_y: 12.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 4800.0,
            feedrate_draw: 1500.0,
            pen_mechanism: PenMechanism::ZAxis {
                z_safe: 2.5,
                z_draw: 0.2,
                feedrate_z: 700.0,
            },
            klipper_url: "http://klipper.local".to_string(),
            start_gcode: "G21\nG90\nG28\n".to_string(),
            end_gcode: "M400\nG0 Z10 F1000\nG28 X Y\nM84\n".to_string(),
        }
    }

    pub fn ender_3() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Creality Ender 3 / V2 / S1".to_string(),
            bed_width: 220.0,
            bed_height: 220.0,
            toolhead_offset_x: -40.0,
            toolhead_offset_y: 10.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 3000.0,
            feedrate_draw: 1200.0,
            pen_mechanism: PenMechanism::default(),
            klipper_url: "http://klipper.local".to_string(),
            start_gcode: "G21 ; mm\nG90 ; absolute coordinates\nG28 ; home all\n".to_string(),
            end_gcode: "M400 ; wait for moves\nG0 Z10 F1000 ; lift pen\nG28 X Y ; park carriage\nM84 ; disable motors\n".to_string(),
        }
    }

    pub fn cr_10() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Creality CR-10 / Ender 3 Max".to_string(),
            bed_width: 300.0,
            bed_height: 300.0,
            toolhead_offset_x: -40.0,
            toolhead_offset_y: 10.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 3000.0,
            feedrate_draw: 1200.0,
            pen_mechanism: PenMechanism::default(),
            klipper_url: "http://klipper.local".to_string(),
            start_gcode: "G21\nG90\nG28\n".to_string(),
            end_gcode: "M400\nG0 Z10 F1000\nG28 X Y\nM84\n".to_string(),
        }
    }

    pub fn bambu_lab_series() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Bambu Lab X1 / P1P / P1S / A1".to_string(),
            bed_width: 256.0,
            bed_height: 256.0,
            toolhead_offset_x: -25.0,
            toolhead_offset_y: 15.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 9000.0,
            feedrate_draw: 2400.0,
            pen_mechanism: PenMechanism::ZAxis {
                z_safe: 2.0,
                z_draw: 0.2,
                feedrate_z: 800.0,
            },
            klipper_url: "http://bambu.local".to_string(),
            start_gcode: "G21 ; mm\nG90\nG28\n".to_string(),
            end_gcode: "M400\nG0 Z15 F1200\nG28 X Y\nM84\n".to_string(),
        }
    }

    pub fn prusa_mk4() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Prusa i3 MK3S+ / MK4".to_string(),
            bed_width: 250.0,
            bed_height: 210.0,
            toolhead_offset_x: -30.0,
            toolhead_offset_y: 10.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 4500.0,
            feedrate_draw: 1500.0,
            pen_mechanism: PenMechanism::ZAxis {
                z_safe: 2.5,
                z_draw: 0.2,
                feedrate_z: 600.0,
            },
            klipper_url: "http://prusa.local".to_string(),
            start_gcode: "G21\nG90\nG28 W\n".to_string(),
            end_gcode: "M400\nG0 Z10 F1000\nG28 X\nM84\n".to_string(),
        }
    }

    pub fn elegoo_neptune() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Elegoo Neptune 3 / 4".to_string(),
            bed_width: 225.0,
            bed_height: 225.0,
            toolhead_offset_x: -35.0,
            toolhead_offset_y: 10.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 4200.0,
            feedrate_draw: 1500.0,
            pen_mechanism: PenMechanism::default(),
            klipper_url: "http://neptune.local".to_string(),
            start_gcode: "G21\nG90\nG28\n".to_string(),
            end_gcode: "M400\nG0 Z10 F1000\nG28 X Y\nM84\n".to_string(),
        }
    }

    pub fn voron_300() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Voron 2.4 / Trident 300".to_string(),
            bed_width: 300.0,
            bed_height: 300.0,
            toolhead_offset_x: 28.0,
            toolhead_offset_y: -12.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 7000.0,
            feedrate_draw: 2000.0,
            pen_mechanism: PenMechanism::KlipperMacros {
                macro_up: "PEN_UP".to_string(),
                macro_down: "PEN_DOWN".to_string(),
            },
            klipper_url: "http://voron.local".to_string(),
            start_gcode: "G21 ; mm\nG90 ; absolute\nG28 ; home\nSTART_PLOT\n".to_string(),
            end_gcode: "END_PLOT\nM84\n".to_string(),
        }
    }

    pub fn voron_350() -> Self {
        Self {
            is_first_run: false,
            language: Language::Russian,
            name: "Voron 2.4 / Trident 350".to_string(),
            bed_width: 350.0,
            bed_height: 350.0,
            toolhead_offset_x: 28.0,
            toolhead_offset_y: -12.0,
            pen_tip_diameter_mm: 0.5,
            feedrate_travel: 7000.0,
            feedrate_draw: 2000.0,
            pen_mechanism: PenMechanism::KlipperMacros {
                macro_up: "PEN_UP".to_string(),
                macro_down: "PEN_DOWN".to_string(),
            },
            klipper_url: "http://voron.local".to_string(),
            start_gcode: "G21 ; mm\nG90 ; absolute\nG28 ; home\nSTART_PLOT\n".to_string(),
            end_gcode: "END_PLOT\nM84\n".to_string(),
        }
    }

    pub fn presets() -> Vec<PrinterProfile> {
        vec![
            Self::anycubic_kobra_1(),
            Self::anycubic_kobra_2(),
            Self::ender_3(),
            Self::cr_10(),
            Self::bambu_lab_series(),
            Self::prusa_mk4(),
            Self::elegoo_neptune(),
            Self::voron_300(),
            Self::voron_350(),
        ]
    }

    fn config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "bedplotter", "BedPlotter")?;
        Some(proj_dirs.config_dir().join("printer_profile.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut profile) = serde_json::from_str::<PrinterProfile>(&content) {
                        profile.name = profile.clean_name();
                        return profile;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)? ;
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_exist() {
        let presets = PrinterProfile::presets();
        assert!(presets.len() >= 9, "Must have at least 9 printer presets");
        assert!(presets.iter().any(|p| p.name.contains("Bambu Lab")));
        assert!(presets.iter().any(|p| p.name.contains("Voron")));
        assert!(presets.iter().any(|p| p.name.contains("Ender")));
        assert!(presets.iter().any(|p| p.name.contains("Prusa")));
        assert!(presets.iter().any(|p| p.name.contains("Neptune")));
    }
}
