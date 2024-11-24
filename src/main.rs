#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod shader;

use std::{env, path::PathBuf};

use eframe::egui;
use shader::{lava_lamp, Shaders};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OpenRgb shader controller",
        options,
        Box::new(|_| Ok(Box::<App>::default())),
    )
}

struct App {
    shaders: Shaders,
    selected_index: Option<u8>,
    profile_path: PathBuf,
}

impl Default for App {
    fn default() -> Self {
        let mut profile_path: PathBuf = env::var("HOME").unwrap().into();
        profile_path.push(".config/OpenRGB/plugins/settings/effect-profiles/shader-controller");
        Self {
            shaders: Shaders::default(),
            selected_index: None,
            profile_path,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            if ui.button("import").clicked() {
                self.shaders.parse_from_profile(self.profile_path.clone());
            }

            for (index, shader) in self.shaders.iter_mut().enumerate() {
                let color = if self
                    .selected_index
                    .is_some_and(|selected_index| selected_index == index as u8)
                {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::GRAY
                };

                let button = egui::Button::new(shader.device_names()[0].clone()).fill(color);
                if ui.add(button).clicked() {
                    self.selected_index = Some(index as u8);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected_index) = self.selected_index {
                let selected_shader = self.shaders.get_shader(selected_index as usize);

                // this is a place holder. parser should not be public
                if selected_shader.parser.is_none() {
                    selected_shader.parser = Some(Box::new(lava_lamp::LavaLampParser::default()));
                }

                selected_shader.settings_ui(ui);
            }
        });
    }
}
