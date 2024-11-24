use eframe::egui;
use rand::Rng;
use regex::Regex;

use crate::utils::float_to_string_decimal;

use super::ShaderParser;

#[derive(Debug, Default, Clone, PartialEq)]
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
        } else {
            self.blobs.push(Blob::default())
        }
    }

    fn get_random_blob(&self) -> Blob {
        let mut rng = rand::thread_rng();
        let num_blobs = self.blobs.len();
        let index = rng.gen_range(0..num_blobs);

        self.blobs[index]
    }

    fn parse_blob_definition(&self, line: &str) -> Option<Blob> {
        let re = regex::Regex::new(r"Blob\s*\(\s*vec2\([^)]+\),\s*vec3\(([^)]+)\),\s*([\d.]+),\s*([\d.]+),\s*([\d.]+)\s*\)").unwrap();

        if let Some(captures) = re.captures(line) {
            let color_str = &captures[1];
            let size = captures[2].parse::<f32>().ok()?;
            let speed = captures[3].parse::<f32>().ok()?;
            let smoothness = captures[4].parse::<f32>().ok()?;

            let color: Vec<f32> = color_str
                .split(',')
                .filter_map(|s| s.trim().parse::<f32>().ok())
                .collect();

            if color.len() == 3 {
                return Some(Blob {
                    color: [color[0], color[1], color[2]],
                    size,
                    speed,
                    smoothness,
                });
            }
        }

        None
    }
}

impl ShaderParser for LavaLampParser {
    fn settings_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        let prev_state = self.clone();
        let mut delete_index = None;
        for (index, blob) in self.blobs.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut blob.color);

                ui.label("size:");
                ui.add(egui::Slider::new(&mut blob.size, 0.0..=100.0));

                ui.label("speed");
                ui.add(egui::Slider::new(&mut blob.speed, 0.0..=100.0));

                ui.label("smoothness:");
                ui.add(egui::Slider::new(&mut blob.smoothness, 0.0..=25.0));

                if ui.button("delete").clicked() {
                    delete_index = Some(index);
                }
            });
        }

        if let Some(index) = delete_index {
            self.blobs.remove(index);
        }

        if ui.button("add blob").clicked() {
            self.add_blob();
        }

        return prev_state != *self;
    }

    fn parse(&mut self, shader_string: &str) {
        self.blobs.clear();

        for line in shader_string.lines() {
            if let Some(blob) = self.parse_blob_definition(line) {
                self.blobs.push(blob);
            }
        }
    }

    fn export(&self, shader_string: &str) -> String {
        let num_blobs = self.blobs.len().to_string();

        let num_blobs_regex = Regex::new(r"#define numBlobs \d+").unwrap();
        let updated_shader =
            num_blobs_regex.replace_all(shader_string, &format!("#define numBlobs {}", num_blobs));

        // Regex for matching the initializeBlobs() function call and replacing it with blob definitions
        let initialize_blobs_regex = Regex::new(r"void initializeBlobs\(\) \{[^\}]*\}").unwrap();

        // Replace the initializeBlobs function with a new function that sets up the blobs from self.blobs
        let final_shader =
            initialize_blobs_regex.replace_all(&updated_shader, |_caps: &regex::Captures| {
                let mut blob_initializations = String::new();
                for (i, blob) in self.blobs.iter().enumerate() {
                    let color = format!(
                        "vec3({}, {}, {})",
                        float_to_string_decimal(blob.color[0]),
                        float_to_string_decimal(blob.color[1]),
                        float_to_string_decimal(blob.color[2]),
                    );
                    blob_initializations.push_str(&format!(
                        "    blobs[{}] = Blob(vec2(0), {}, {}, {}, {});\n",
                        i,
                        color,
                        float_to_string_decimal(blob.size),
                        float_to_string_decimal(blob.speed),
                        float_to_string_decimal(blob.smoothness),
                    ));
                }
                format!("void initializeBlobs() {{\n{}}}", blob_initializations)
            });
        final_shader.to_string()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Blob {
    color: [f32; 3],
    size: f32,
    speed: f32,
    smoothness: f32,
}
