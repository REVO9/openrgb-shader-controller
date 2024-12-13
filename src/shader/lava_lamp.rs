use std::default;

use eframe::egui;
use rand::Rng;
use regex::Regex;
use undo::{Edit, Record};

use crate::utils::float_to_string_decimal;

use super::{RecordWrap, ShaderParser};

#[derive(Clone, Debug, PartialEq)]
pub struct LavaLampParser {
    blobs: Blobs,
    record_wrap: RecordWrap<BlobsEdit>,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Blobs(Vec<Blob>);

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Blob {
    color: [f32; 3],
    size: f32,
    speed: f32,
    smoothness: f32,
}

impl Blobs {
    fn get_random_prop_blob(&self) -> Blob {
        Blob {
            color: self.get_random_blob().color,
            size: self.get_random_blob().size,
            speed: self.get_random_blob().speed,
            smoothness: self.get_random_blob().smoothness,
        }
    }

    fn get_random_blob(&self) -> Blob {
        let mut rng = rand::thread_rng();
        let num_blobs = self.0.len();
        let index = rng.gen_range(0..num_blobs);

        self.0[index]
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

impl Default for LavaLampParser {
    fn default() -> Self {
        Self {
            blobs: Blobs::default(),
            record_wrap: RecordWrap(Record::new()),
        }
    }
}

impl ShaderParser for LavaLampParser {
    fn settings_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        let prev_state = self.clone();
        for (index, blob) in self.blobs.0.clone().iter().enumerate() {
            let mut prop_blob = blob.clone();
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut prop_blob.color);

                ui.label("size:");
                ui.add(egui::Slider::new(&mut prop_blob.size, 0.0..=100.0));

                ui.label("speed");
                ui.add(egui::Slider::new(&mut prop_blob.speed, 0.0..=100.0));

                ui.label("smoothness:");
                ui.add(egui::Slider::new(&mut prop_blob.smoothness, 0.0..=25.0));

                if ui.button("delete").clicked() {
                    self.record_wrap
                        .0
                        .edit(&mut self.blobs, BlobsEdit::DeleteBlob(index, *blob))
                }
            });
            if prop_blob != *blob {
                self.record_wrap.0.edit(
                    &mut self.blobs,
                    BlobsEdit::MutateBlob {
                        index,
                        old: *blob,
                        new: prop_blob,
                    },
                )
            }
        }

        if ui.button("add blob").clicked() {
            let random_blob = self.blobs.get_random_blob();
            self.record_wrap
                .0
                .edit(&mut self.blobs, BlobsEdit::AddBlob(random_blob));
        }

        return prev_state != *self;
    }

    fn parse(&mut self, shader_string: &str) {
        self.blobs.0.clear();

        for line in shader_string.lines() {
            if let Some(blob) = self.blobs.parse_blob_definition(line) {
                self.blobs.0.push(blob);
            }
        }
    }

    fn export(&self, shader_string: &str) -> String {
        let num_blobs = self.blobs.0.len().to_string();

        let num_blobs_regex = Regex::new(r"#define numBlobs \d+").unwrap();
        let updated_shader =
            num_blobs_regex.replace_all(shader_string, &format!("#define numBlobs {}", num_blobs));

        // Regex for matching the initializeBlobs() function call and replacing it with blob definitions
        let initialize_blobs_regex = Regex::new(r"void initializeBlobs\(\) \{[^\}]*\}").unwrap();

        // Replace the initializeBlobs function with a new function that sets up the blobs from self.blobs
        let final_shader =
            initialize_blobs_regex.replace_all(&updated_shader, |_caps: &regex::Captures| {
                let mut blob_initializations = String::new();
                for (i, blob) in self.blobs.0.iter().enumerate() {
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

    fn undo(&mut self) {
        self.record_wrap.0.undo(&mut self.blobs);
    }

    fn redo(&mut self) {
        self.record_wrap.0.redo(&mut self.blobs);
    }
}

#[derive(Clone, Copy)]
enum BlobsEdit {
    AddBlob(Blob),
    DeleteBlob(usize, Blob),
    MutateBlob { index: usize, old: Blob, new: Blob },
}

impl Edit for BlobsEdit {
    type Target = Blobs;

    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            BlobsEdit::AddBlob(blob) => {
                target.0.push(*blob);
            }
            BlobsEdit::MutateBlob { index, old, new } => {
                let list_blob = target.0.get_mut(*index).unwrap();
                *old = *list_blob;
                *list_blob = *new;
            }
            BlobsEdit::DeleteBlob(index, blob) => {
                *blob = target.0.remove(*index);
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            BlobsEdit::AddBlob(blob) => {
                *blob = target.0.pop().unwrap();
            }
            BlobsEdit::MutateBlob { index, old, .. } => {
                let list_blob = target.0.get_mut(*index).unwrap();
                *list_blob = *old;
            }
            BlobsEdit::DeleteBlob(index, blob) => {
                target.0.insert(*index, *blob);
            }
        }
    }
}
