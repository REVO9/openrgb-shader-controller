use eframe::egui;
use rand::Rng;

use super::ShaderParser;

#[derive(Debug, Default)]
pub struct LavaLampParser {
    blobs: Vec<Blob>,
}

impl LavaLampParser {
    fn add_blob(&mut self) {
        if !self.blobs.is_empty() {
            self.blobs.push(Blob {
                color: self.get_random_blob().color,
                size: self.get_random_blob().size,
                speed: self.get_random_blob().speed,
                smoothness: self.get_random_blob().smoothness,
            })
        }
        else {
            self.blobs.push(Blob::default())
        }
    }

    fn get_random_blob(&self) -> Blob {
        let mut rng = rand::thread_rng();
        let num_blobs = self.blobs.len();
        let index = rng.gen_range(0..num_blobs);

        self.blobs[index]
    }
}

impl ShaderParser for LavaLampParser {
    fn settings_ui(&mut self, ui: &mut eframe::egui::Ui) {
        for blob in self.blobs.iter_mut() {
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut blob.color);

                ui.label("size:");
                ui.add(egui::Slider::new(&mut blob.size, 0.0..=100.0));

                ui.label("speed");
                ui.add(egui::Slider::new(&mut blob.speed, 0.0..=100.0));

                ui.label("smoothness:");
                ui.add(egui::Slider::new(&mut blob.smoothness, 0.0..=100.0));
            });
        }

        if ui.button("add blob").clicked() {
            self.add_blob();
        }
    }

    fn parse(&mut self, string: &str) {
        todo!()
    }

    fn export(&self) -> String {
        todo!()
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Blob {
    color: [f32; 3],
    size: f32,
    speed: f32,
    smoothness: f32,
}
