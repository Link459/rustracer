use anyhow::Result;
use image::{open, ExtendedColorType};
use rand::{thread_rng, Rng};
use std::{
    fs::{self, File},
    io::Write,
    time::{Duration, Instant},
};

use crate::{
    camera::Camera,
    hittable::Hittable,
    image::Image,
    render::{Background, RenderConfig},
    scene::Scene,
    texture::{ImageTexture, TextureStorage},
};

pub fn serialize_scene(scene: &Scene, path: &str) -> Result<()> {
    let extensions = ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES;
    let config = ron::ser::PrettyConfig::new()
        .struct_names(false)
        .extensions(extensions);
    let data = ron::ser::to_string_pretty(&scene, config)?;
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    return Ok(());
}

pub fn deserialize_scene(path: &str) -> Result<Scene> {
    let data = fs::read_to_string(path)?;
    let world = ron::from_str::<Scene>(&data)?;
    return Ok(world);
}

pub fn get_time_prediction(rays: u32, camera: &Camera, _world: &impl Hittable) -> Duration {
    let width = camera.get_config().width;
    let height = camera.get_config().height;
    let samples = 100;

    let mut rng = thread_rng();
    let mut elapsed = Duration::default();
    for _ in 0..samples {
        let _w = rng.gen_range(0..width);
        let _h = rng.gen_range(0..height);
        let time = Instant::now();
        //TODO: fix this
        //camera.trace_ray(w, h, world);
        elapsed += time.elapsed();
    }
    let average = elapsed / samples;
    let total_ray = average * rays;
    let with_threading = total_ray / rayon::current_num_threads() as u32;
    return with_threading;
}

pub fn number_with_decimals(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

pub fn create_ppm_file(file: &str, buf: &[u8], width: u32, height: u32) -> Result<()> {
    let mut file = File::create(file)?;
    let ppm = format!("P6\n {:?} {:?}\n255\n", width, height);
    file.write_all(ppm.as_bytes())?;
    file.write_all(buf)?;
    Ok(())
}

pub fn create_image_file(file: &str, buf: &[u8], width: u32, height: u32) -> Result<()> {
    image::save_buffer(file, buf, width, height, ExtendedColorType::Rgb8)?;
    Ok(())
}

pub fn load_hdri(path: &str) -> Result<Image> {
    let hdri = open(path)?;
    let hdri = hdri.into_rgb8();
    return Ok(Image::from(hdri));
}

pub fn linear_plane_index(len: usize, width: u32, row: u32, column: u32) -> usize {
    return len - (row * width + column) as usize;
}

pub fn parse_render_settings(options: &[String], mut orig: RenderConfig) -> RenderConfig {
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
