use std::path::PathBuf;

use crate::{
    render::{Background, RenderSettings},
    texture::{ImageTexture, TextureStorage},
};

pub struct Settings {
    pub output: PathBuf,
    pub render_settings: RenderSettings,
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
            "--background" => match option_value[1].as_str() {
                "Night" => {
                    orig.background = Background::Night;
                }
                "Sky" => {
                    orig.background = Background::Sky;
                }
                x => {
                    orig.background = Background::Hdri(TextureStorage::Image(ImageTexture::new(x)));
                }
            },
            _ => {}
        };
    }
    return orig;
}
