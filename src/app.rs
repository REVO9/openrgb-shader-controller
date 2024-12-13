use std::{
    env,
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::egui::{self, Key};

use crate::shader::{lava_lamp, Shaders};

pub struct App {
    shaders: Shaders,
    selected_index: Option<u8>,
    profile_path: PathBuf,
    last_change: Instant,
    should_reload: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut profile_path: PathBuf = env::var("HOME").unwrap().into();
        profile_path.push(".config/OpenRGB/plugins/settings/effect-profiles/shader-controller");
        let mut shaders = Shaders::default();
        shaders.parse_from_profile(profile_path.clone());

        Self {
            shaders,
            selected_index: None,
            profile_path,
            last_change: Instant::now(),
            should_reload: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            for (index, shader) in self.shaders.iter_mut().enumerate() {
                let color = if self
                    .selected_index
                    .is_some_and(|selected_index| selected_index == index as u8)
                {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::GRAY
                };

                let shader_name = shader.name().to_string().clone();
                let button_label = shader.device_names().first().unwrap_or(&shader_name);
                let button = egui::Button::new(button_label).fill(color);
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
                    selected_shader.parse();
                }

                if ctx.input(|i| i.key_pressed(Key::Z) && i.modifiers.command_only()) {
                    selected_shader.undo();
                }

                if ctx.input(|i| i.key_pressed(Key::Y) && i.modifiers.command_only()) {
                    selected_shader.redo();
                }

                let changed = selected_shader.settings_ui(ui);
                if changed {
                    self.last_change = Instant::now();
                    self.should_reload = true
                }
                if self.last_change.elapsed() > Duration::from_millis(100) && self.should_reload {
                    self.should_reload = false;
                    println!("reloading");
                    selected_shader.export();
                    self.shaders.save_to_profile(self.profile_path.clone());
                    Shaders::reload_openrgb();
                }
            }
        });
    }
}
