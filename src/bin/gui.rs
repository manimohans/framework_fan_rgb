use eframe::egui;
use framework_lib::chromium_ec::CrosEcDriverType;
use framework_lib::chromium_ec::commands::RgbS;
use rand::Rng;

use fwd_rgb::{apply_colors, format_ec_error, rgb_from_u32, rgb_to_hex_string};

const COLOR_COUNT: usize = 8;
const PRESET_SPECTRUM: [u32; COLOR_COUNT] = [
    0xFF0000, 0xFF7F00, 0xFFFF00, 0x00FF00, 0x0000FF, 0x4B0082, 0x9400D3, 0xFFFFFF,
];
const PRESET_EMBER: u32 = 0xF2662B;
const PRESET_MATRIX: [u32; COLOR_COUNT] = [
    0x00FF66, 0x00CC44, 0x009933, 0x006622, 0x00FF99, 0x00CC66, 0x009944, 0x006633,
];
const PRESET_AZURE: [u32; COLOR_COUNT] = [
    0x0C0CFF, 0x1A1AFF, 0x2B2BFF, 0x3C3CFF, 0x1A4FFF, 0x2E5FFF, 0x4370FF, 0x5880FF,
];
const PRESET_VOID: [u32; COLOR_COUNT] = [
    0x050505, 0x0A0A0A, 0x101010, 0x161616, 0x101010, 0x0A0A0A, 0x050505, 0x080808,
];
const PRESET_NEON_CITY: [u32; COLOR_COUNT] = [
    0xFF00FF, 0x00FFFF, 0x9400D3, 0xFF0099, 0x00CCFF, 0x8A2BE2, 0xFF1493, 0x00BFFF,
];
const PRESET_SOLAR_FLARE: [u32; COLOR_COUNT] = [
    0xFF4500, 0xFF8C00, 0xFFA500, 0xFFD700, 0xFF6347, 0xFF7F50, 0xFFD700, 0xFFFF00,
];
const PRESET_ABYSS: [u32; COLOR_COUNT] = [
    0x000080, 0x00008B, 0x191970, 0x0000CD, 0x4169E1, 0x0000FF, 0x1E90FF, 0x00BFFF,
];
const PRESET_CANOPY: [u32; COLOR_COUNT] = [
    0x006400, 0x228B22, 0x32CD32, 0x90EE90, 0x008000, 0x6B8E23, 0x556B2F, 0x8FBC8F,
];
const PRESET_DREAMSCAPE: [u32; COLOR_COUNT] = [
    0xFFB6C1, 0xFF69B4, 0xE6E6FA, 0xD8BFD8, 0xDDA0DD, 0xEE82EE, 0xFFC0CB, 0xFFA07A,
];

fn color32_from_rgb(color: RgbS) -> egui::Color32 {
    egui::Color32::from_rgb(color.r, color.g, color.b)
}

fn rgb_from_color32(color: egui::Color32) -> RgbS {
    RgbS {
        r: color.r(),
        g: color.g(),
        b: color.b(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DriverChoice {
    Auto,
    Portio,
    CrosEc,
    Windows,
}

impl DriverChoice {
    fn to_option(self) -> Option<CrosEcDriverType> {
        match self {
            DriverChoice::Auto => None,
            DriverChoice::Portio => Some(CrosEcDriverType::Portio),
            DriverChoice::CrosEc => Some(CrosEcDriverType::CrosEc),
            DriverChoice::Windows => Some(CrosEcDriverType::Windows),
        }
    }

    fn label(self) -> &'static str {
        match self {
            DriverChoice::Auto => "Auto (recommended)",
            DriverChoice::Portio => "Port I/O",
            DriverChoice::CrosEc => "Linux cros_ec",
            DriverChoice::Windows => "Windows HID",
        }
    }

    fn all() -> &'static [DriverChoice] {
        &[
            DriverChoice::Auto,
            DriverChoice::Portio,
            DriverChoice::CrosEc,
            DriverChoice::Windows,
        ]
    }
}

#[derive(Clone, Copy)]
enum StatusKind {
    Info,
    Success,
    Error,
}

struct StatusMessage {
    kind: StatusKind,
    text: String,
}

struct FanRgbApp {
    start_key: u8,
    colors: [egui::Color32; COLOR_COUNT],
    status: Option<StatusMessage>,
    driver: DriverChoice,
    auto_apply: bool,
    dirty: bool,
    lights_enabled: bool,
}

impl FanRgbApp {
    fn new() -> Self {
        let colors = PRESET_SPECTRUM
            .iter()
            .map(|color| color32_from_rgb(rgb_from_u32(*color)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or([egui::Color32::BLACK; COLOR_COUNT]);

        Self {
            start_key: 0,
            colors,
            status: Some(StatusMessage {
                kind: StatusKind::Info,
                text: "Adjust the colors and press Apply to update the fan LEDs.".to_string(),
            }),
            driver: DriverChoice::Auto,
            auto_apply: false,
            dirty: false,
            lights_enabled: true,
        }
    }

    fn set_status(&mut self, kind: StatusKind, text: impl Into<String>) {
        self.status = Some(StatusMessage {
            kind,
            text: text.into(),
        });
    }

    fn current_colors(&self) -> Vec<RgbS> {
        self.colors.iter().copied().map(rgb_from_color32).collect()
    }

    fn apply_palette(&mut self, palette: &[u32]) {
        for idx in 0..COLOR_COUNT {
            let value = palette[idx % palette.len()];
            self.colors[idx] = color32_from_rgb(rgb_from_u32(value));
        }
        self.dirty = true;
        self.lights_enabled = true;
    }

    fn finish_preset(&mut self) {
        if self.auto_apply && self.lights_enabled {
            self.apply();
        }
    }

    fn apply(&mut self) {
        if !self.lights_enabled {
            match self.turn_off_lights() {
                Ok(message) => {
                    self.dirty = false;
                    self.set_status(StatusKind::Info, message);
                }
                Err(err) => {
                    self.set_status(StatusKind::Error, err);
                }
            }
            return;
        }

        match apply_colors(
            self.start_key,
            self.current_colors(),
            self.driver.to_option(),
        ) {
            Ok(()) => {
                self.dirty = false;
                self.set_status(
                    StatusKind::Success,
                    format!(
                        "Updated {} colors starting at {}",
                        COLOR_COUNT, self.start_key
                    ),
                );
            }
            Err(err) => {
                self.set_status(StatusKind::Error, format_ec_error(&err));
            }
        }
    }

    fn turn_off_lights(&mut self) -> Result<String, String> {
        let off = vec![RgbS { r: 0, g: 0, b: 0 }; COLOR_COUNT];
        apply_colors(self.start_key, off, self.driver.to_option())
            .map(|_| "Fan lighting disabled".to_string())
            .map_err(|err| format_ec_error(&err))
    }

    fn reset_spectrum(&mut self) {
        self.apply_palette(&PRESET_SPECTRUM);
    }

    fn apply_entropy(&mut self) {
        let mut rng = rand::thread_rng();
        self.colors = std::array::from_fn(|_| {
            egui::Color32::from_rgb(
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
            )
        });
        self.dirty = true;
        self.lights_enabled = true;
    }

    fn apply_twilight(&mut self) {
        let first = self.colors.first().copied().unwrap_or(egui::Color32::BLACK);
        let last = self.colors.last().copied().unwrap_or(egui::Color32::BLACK);

        for (idx, color) in self.colors.iter_mut().enumerate() {
            let t = idx as f32 / (COLOR_COUNT.saturating_sub(1) as f32).max(1.0);
            *color = egui::Color32::from_rgb(
                Self::lerp_channel(first.r(), last.r(), t),
                Self::lerp_channel(first.g(), last.g(), t),
                Self::lerp_channel(first.b(), last.b(), t),
            );
        }

        self.dirty = true;
        self.lights_enabled = true;
    }

    fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
        let a = a as f32;
        let b = b as f32;
        (a + (b - a) * t).round().clamp(0.0, 255.0) as u8
    }

    fn apply_ember(&mut self) {
        for color in &mut self.colors {
            *color = color32_from_rgb(rgb_from_u32(PRESET_EMBER));
        }
        self.lights_enabled = true;
        self.dirty = true;
    }
}

impl eframe::App for FanRgbApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Framework CPU Fan RGB");
            ui.label("Configure up to 8 contiguous zones and apply them directly via the EC.");
            ui.add(
                egui::Label::new(
                    "Note: EC access requires administrative privileges (sudo or \
Administrator) on Framework systems.",
                )
                .wrap(true),
            );
        });

        egui::SidePanel::right("presets_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Presets");

                if ui.button("Spectrum").clicked() {
                    self.reset_spectrum();
                    self.finish_preset();
                }
                if ui.button("Ember").clicked() {
                    self.apply_ember();
                    self.finish_preset();
                }
                if ui.button("Twilight").clicked() {
                    self.apply_twilight();
                    self.finish_preset();
                }
                if ui.button("Entropy").clicked() {
                    self.apply_entropy();
                    self.finish_preset();
                }
                if ui.button("Matrix").clicked() {
                    self.apply_palette(&PRESET_MATRIX);
                    self.finish_preset();
                }
                if ui.button("Azure").clicked() {
                    self.apply_palette(&PRESET_AZURE);
                    self.finish_preset();
                }
                if ui.button("Void").clicked() {
                    self.apply_palette(&PRESET_VOID);
                    self.finish_preset();
                }
                if ui.button("Neon City").clicked() {
                    self.apply_palette(&PRESET_NEON_CITY);
                    self.finish_preset();
                }
                if ui.button("Solar Flare").clicked() {
                    self.apply_palette(&PRESET_SOLAR_FLARE);
                    self.finish_preset();
                }
                if ui.button("Abyss").clicked() {
                    self.apply_palette(&PRESET_ABYSS);
                    self.finish_preset();
                }
                if ui.button("Canopy").clicked() {
                    self.apply_palette(&PRESET_CANOPY);
                    self.finish_preset();
                }
                if ui.button("Dreamscape").clicked() {
                    self.apply_palette(&PRESET_DREAMSCAPE);
                    self.finish_preset();
                }

                ui.separator();
                ui.checkbox(&mut self.auto_apply, "Auto-apply after changes");
                if ui
                    .checkbox(&mut self.lights_enabled, "Lighting enabled")
                    .changed()
                {
                    self.apply();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Start key");
                ui.add(
                    egui::Slider::new(&mut self.start_key, 0..=255)
                        .text("index")
                        .clamp_to_range(true),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Driver");
                egui::ComboBox::from_id_source("driver_choice")
                    .selected_text(self.driver.label())
                    .show_ui(ui, |ui| {
                        for choice in DriverChoice::all() {
                            ui.selectable_value(&mut self.driver, *choice, choice.label());
                        }
                    });
            });

            ui.separator();
            ui.heading("Colors");

            for idx in 0..COLOR_COUNT {
                let mut color_value = self.colors[idx];
                let mut updated = false;

                ui.horizontal(|ui| {
                    ui.label(format!("Zone {}", idx + 1));
                    let mut egui_color = color_value;
                    let response = egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut egui_color,
                        egui::color_picker::Alpha::Opaque,
                    );

                    if response.changed() {
                        color_value = egui_color;
                        updated = true;
                    }

                    ui.label(rgb_to_hex_string(rgb_from_color32(color_value)));
                });

                if updated {
                    self.colors[idx] = color_value;
                    self.dirty = true;
                    self.lights_enabled = true;
                    if self.auto_apply {
                        self.apply();
                    }
                } else {
                    self.colors[idx] = color_value;
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    self.apply();
                }

                let toggle_label = if self.lights_enabled {
                    "Turn off"
                } else {
                    "Turn on"
                };
                if ui.button(toggle_label).clicked() {
                    self.lights_enabled = !self.lights_enabled;
                    self.apply();
                }

                if ui.button("Reset unsaved changes").clicked() {
                    self.reset_spectrum();
                }
            });

            if let Some(status) = &self.status {
                ui.separator();
                let color = match status.kind {
                    StatusKind::Info => egui::Color32::LIGHT_GRAY,
                    StatusKind::Success => egui::Color32::from_rgb(0, 200, 83),
                    StatusKind::Error => egui::Color32::from_rgb(209, 71, 78),
                };
                ui.colored_label(color, &status.text);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([660.0, 480.0])
            .with_min_inner_size([520.0, 360.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Framework Fan RGB",
        native_options,
        Box::new(|_cc| Box::new(FanRgbApp::new())),
    )
}
