#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod shader;

use eframe::egui;
use shader::Shaders;

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
}

impl Default for App {
    fn default() -> Self {
        Self {
            shaders: Shaders::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
