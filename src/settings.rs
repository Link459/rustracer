use std::{
    path::{ PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{
    render::{RenderSettings, Skybox},
    texture::{ImageTexture, TextureStorage},
};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum PresentSettings {
    #[default]
    OnceDone,
    Accumulate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SceneSettings {
    Index(usize),
    Path(PathBuf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub output: PathBuf,
    pub present_settings: PresentSettings,
    pub log_messages: bool,
    pub render_settings: RenderSettings,
    pub scene_settings: SceneSettings,
}

impl Settings {
    pub fn new(render_settings: RenderSettings) -> Self {
        return Self {
            output: PathBuf::from("out/"),
            present_settings: PresentSettings::OnceDone,
            log_messages: true,
            render_settings,
            scene_settings: SceneSettings::Index(0),
        };
    }

    pub fn parse(options: &[String], render_settings: RenderSettings) -> Self {
        let mut settings = Self::new(render_settings);

        for option_value in options.chunks(2) {
            let get_val = || option_value[1].parse::<u32>().unwrap();
            match option_value[0].as_str() {
                "--settings" => {
                    let path = &option_value[1];
                    let data = std::fs::read_to_string(path).unwrap();
                    let settings = toml::from_str::<Settings>(&data).unwrap();
                    return settings;
                }
                "--scene" => {
                    if let Ok(idx) = option_value[1].parse::<usize>() {
                        settings.scene_settings = SceneSettings::Index(idx)
                    } else {
                        let path = &option_value[1];
                        settings.scene_settings =
                            SceneSettings::Path(PathBuf::from_str(path).unwrap());
                    }
                }
                "--samples" => {
                    settings.render_settings.samples = get_val();
                }
                "--width" => {
                    settings.render_settings.width = get_val();
                }
                "--height" => {
                    settings.render_settings.height = get_val();
                }
                "--present" => match option_value[1].as_str() {
                    "once" => {
                        settings.present_settings = PresentSettings::OnceDone;
                    }
                    "accumulate" => {
                        settings.present_settings = PresentSettings::Accumulate;
                    }
                    _ => {}
                },
                "--background" => match option_value[1].as_str() {
                    "Night" => {
                        settings.render_settings.skybox = Skybox::Night;
                    }
                    "Sky" => {
                        settings.render_settings.skybox = Skybox::Sky;
                    }
                    x => {
                        settings.render_settings.skybox =
                            Skybox::Hdri(TextureStorage::Image(ImageTexture::new(x)));
                    }
                },
                _ => {}
            };
        }
        return settings;
    }
}

impl Default for Settings {
    fn default() -> Self {
        return Self {
            output: PathBuf::from("out/"),
            present_settings: PresentSettings::OnceDone,
            log_messages: false,
            render_settings: RenderSettings::default(),
            scene_settings: SceneSettings::Index(0),
        };
    }
}

fn parse_settings(mut settings: Settings, args: &[String]) -> Settings {
    settings.render_settings = parse_render_settings(args, settings.render_settings);
    return settings;
}

pub fn parse_render_settings(options: &[String], mut orig: RenderSettings) -> RenderSettings {
    for option_value in options.chunks(2) {
        let get_val = || option_value[1].parse::<u32>().unwrap();
        match option_value[0].as_str() {
            "--samples" => {
                orig.samples = get_val();
            }
            "--width" => {
                orig.width = get_val();
            }
            "--height" => {
                orig.height = get_val();
            }
            "--present" => match option_value[1].as_str() {
                "once" => {}
                "accumulate" => {}
                _ => {}
            },
            "--background" => match option_value[1].as_str() {
                "Night" => {
                    orig.skybox = Skybox::Night;
                }
                "Sky" => {
                    orig.skybox = Skybox::Sky;
                }
                x => {
                    orig.skybox = Skybox::Hdri(TextureStorage::Image(ImageTexture::new(x)));
                }
            },
            _ => {}
        };
    }
    return orig;
}
