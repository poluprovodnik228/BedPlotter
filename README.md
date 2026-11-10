# 🖋️ BedPlotter — 3D Printer Pen Plotter Studio

<p align="center">
  <b>High-performance Rust desktop CAM software that transforms any FDM 3D printer into a precision pen plotter.</b><br>
  <i>Designed with OrcaSlicer / Bambu Studio dark industrial aesthetics and sub-millimeter precision.</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/GUI-eframe%20%2F%20egui-blue" alt="GUI">
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen" alt="Platform">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
</p>

---

## ✨ Features

- 🖤 **OrcaSlicer / Bambu Studio Aesthetics:** Monolithic dark CAM layout (`#121214`), custom Inter typography, rigid 3-zone header, and streamlined toolbars.
- ✍️ **True Single-line Vector Fonts (WYSIWYG):**
  - **GOST 2.304** technical blueprint lettering.
  - **Handwritten Cursive** with natural continuous connecting strokes.
  - Live real-time typography editing directly on the bed canvas.
- 🎨 **Organic Hand-Drawn Humanizer:** Adds natural micro-jitter and imperfection to simulate genuine human handwriting with configurable jitter strength.
- 📐 **CAD Geometry & Hatch Infill:** 
  - Direct canvas click-to-place tools: Rectangles, Circles, Stars, and Bed Calibration Grids.
  - Interactive bounding box with drag-to-resize handles and aspect-ratio lock 🔗.
  - Linear hatch and cross-hatch shading with customizable density and angle.
- ⚡ **TSP Path Optimization:** Solves the Travelling Salesperson Problem (Nearest Neighbor) to minimize pen-up travel moves, reducing plotting time by up to 40%.
- ⏱️ **G-Code Simulation & Production ETA:**
  - Real-time travel/draw playback simulation with play/pause and progress scrubbing.
  - Accurate metrics: draw distance, travel distance, segment count, and exact time estimates.
- 📷 **Homography Bed Photo Calibration:** 3×3 projective homography transformation (`nalgebra`) with bilinear texture interpolation for 1:1 camera-to-bed alignment.
- 🖨️ **Pre-tuned Printer Profiles:**
  - **Anycubic Kobra 1** (222×222 mm, pre-configured Z-axis / Servo pen offsets).
  - Anycubic Kobra 2 / Neo / Go (220×220 mm).
  - Creality Ender 3 / V2 / S1 (220×220 mm) & CR-10 / Ender 3 Max (300×300 mm).
  - Bambu Lab X1 / P1P / P1S / A1 (256×256 mm).
  - Prusa i3 MK3S+ / MK4 (250×210 mm).
  - Voron 2.4 / Trident (300×300 & 350×350 mm).
- 🌐 **Full Dual-Language Support:** Instant 100% switching between English and Russian without restarts.

---

## 🚀 Quick Start

### Prerequisites
Make sure you have [Rust and Cargo](https://rustup.rs/) installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/<your-username>/bedplotter.git
cd bedplotter

# Run in release mode
cargo run --release
```

### Running Tests

```bash
cargo test
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd + O` / `Ctrl + O` | Import SVG vector file |
| `Cmd + S` / `Ctrl + S` | Export `.gcode` file |
| `V` | Select & Transform tool |
| `T` | Single-line Vector Text tool |
| `R` | Rectangle tool |
| `C` | Circle tool |
| `S` | Star tool |
| `G` | Calibration Grid tool |
| `Delete` / `Backspace` | Delete selected shape (when not typing text) |
| `Esc` | Cancel current tool / Return to Select |

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
