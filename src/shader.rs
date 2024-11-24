pub mod lava_lamp;

use eframe::egui;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub trait ShaderParser
where
    Self: std::fmt::Debug,
{
    fn settings_ui(&mut self, ui: &mut egui::Ui);

    fn parse(&mut self, shader_string: &str);

    fn export(&self, shader_string: &str) -> String;
}

#[derive(Debug)]
pub struct Shader {
    name: String,
    device_names: Vec<String>,
    pub parser: Option<Box<dyn ShaderParser>>,
    // should not be public
    pub shader_str: String,
}

impl Shader {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn device_names(&self) -> &[String] {
        &self.device_names
    }

    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(ref mut parser) = self.parser {
            parser.settings_ui(ui);
        }
    }

    pub fn parse(&mut self) {
        if let Some(ref mut parser) = self.parser {
            print!("parsing shader string: {}", self.shader_str);
            parser.parse(&self.shader_str.replace("\\n", "\n"));
        }
    }

    pub fn export(&mut self) {
        if let Some(ref parser) = self.parser {
            let str = parser.export(&self.shader_str);
            self.shader_str = str;
        }
    }
}

#[derive(Default, Debug)]
pub struct Shaders {
    shaders: Vec<Shader>,
}

impl Shaders {
    pub fn iter(&self) -> impl Iterator<Item = &Shader> {
        self.shaders.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Shader> {
        self.shaders.iter_mut()
    }

    pub fn get_shader(&mut self, index: usize) -> &mut Shader {
        return &mut self.shaders[index];
    }

    pub fn parse_from_profile<T>(&mut self, profile_path: T)
    where
        T: Into<PathBuf>,
    {
        self.shaders.clear();
        let file_content =
            fs::read_to_string(profile_path.into()).expect("Failed to read profile file");
        let json_data: Value = serde_json::from_str(&file_content).expect("Failed to parse JSON");

        let effects = match json_data["Effects"].as_array() {
            Some(effects) => effects,
            None => return,
        };

        for effect in effects {
            let custom_settings = match effect["CustomSettings"].as_object() {
                Some(settings) => settings,
                None => continue,
            };

            let shader_name = match custom_settings["shader_name"].as_str() {
                Some(name) => name,
                None => continue,
            };

            let shader_program = match custom_settings["shader_program"].as_object() {
                Some(program) => program,
                None => continue,
            };

            let main_pass = match shader_program["main_pass"].as_object() {
                Some(pass) => pass,
                None => continue,
            };

            let fragment_shader = match main_pass["fragment_shader"].as_str() {
                Some(shader) => shader,
                None => continue,
            };

            let mut device_names = Vec::new();
            if let Some(controller_zones) = effect["ControllerZones"].as_array() {
                for zone in controller_zones {
                    if let Some(description) = zone["description"].as_str() {
                        device_names.push(description.to_string());
                    }
                }
            }

            // Create and store the Shader object
            self.shaders.push(Shader {
                name: shader_name.to_string(),
                parser: None,
                device_names,
                shader_str: fragment_shader.to_string(),
            });
        }
    }

    pub fn save_to_profile<T>(&self, profile_path: T)
    where
        T: Into<PathBuf>,
    {
        let profile_path = profile_path.into();
        let file_content = fs::read_to_string(&profile_path).expect("Failed to read profile file");

        let mut json_data: Value =
            serde_json::from_str(&file_content).expect("Failed to parse JSON");

        let effects = match json_data["Effects"].as_array_mut() {
            Some(effects) => effects,
            None => return,
        };

        for effect in effects {
            let effect_immutable = effect["ControllerZones"].clone();
            let custom_settings = match effect["CustomSettings"].as_object_mut() {
                Some(settings) => settings,
                None => continue,
            };

            let shader_name = custom_settings["shader_name"].clone();
            let shader_name = match shader_name.as_str() {
                Some(name) => name,
                None => continue,
            };

            let shader_program = match custom_settings["shader_program"].as_object_mut() {
                Some(program) => program,
                None => continue,
            };

            let main_pass = match shader_program["main_pass"].as_object_mut() {
                Some(pass) => pass,
                None => continue,
            };

            let fragment_shader = match main_pass.get_mut("fragment_shader") {
                Some(Value::String(shader)) => shader,
                _ => continue,
            };

            let mut device_names = Vec::new();
            if let Some(controller_zones) = effect_immutable.as_array() {
                for zone in controller_zones {
                    if let Some(description) = zone["description"].as_str() {
                        device_names.push(description.to_string());
                    }
                }
            }

            device_names.sort();
            for shader in self.shaders.iter() {
                let mut sorted_device_names = shader.device_names.clone();
                sorted_device_names.sort();
                if shader.name == shader_name && sorted_device_names == device_names {
                    *fragment_shader = shader.shader_str.clone();
                }
            }
        }

        let updated_content =
            serde_json::to_string_pretty(&json_data).expect("Failed to serialize JSON");
        fs::write(profile_path, updated_content).expect("Failed to write updated profile file");
    }
}
