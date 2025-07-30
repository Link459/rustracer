use anyhow::Result;
use image::open;
use rand::Rng;
use std::{
    fs::{self, File},
    io::Write,
    time::{Duration, Instant},
};

use crate::{
    camera::Camera,
    hittable::Hittable,
    image::Image,
    render::{Background, RenderSettings},
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

pub fn get_time_prediction(rays: u32, _camera: &Camera, _world: &impl Hittable) -> Duration {
    //let width = camera.get_config().width;
    //let height = camera.get_config().height;
    let width = 100;
    let height = 100;
    let samples = 100;

    let mut rng = rand::rng();
    let mut elapsed = Duration::default();
    for _ in 0..samples {
        let _w = rng.random_range(0..width);
        let _h = rng.random_range(0..height);
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

pub fn load_hdri(path: &str) -> Result<Image> {
    let hdri = open(path)?;
    let hdri = hdri.into_rgb32f();
    return Ok(Image::from(hdri));
}

pub fn linear_plane_index(len: usize, width: u32, row: u32, column: u32) -> usize {
    return len - (row * width + column) as usize;
}

pub fn cmd_seperator(name: &str) {
    println!("========{}========", name);
}
