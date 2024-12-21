use anyhow::Result;
use image::{open, ExtendedColorType};
use rand::{thread_rng, Rng};
use std::{
    fs::{self, File},
    io::Write,
    time::{Duration, Instant},
};

use crate::{camera::Camera, hittable::Hittable, image::Image, scene::Scene};

pub fn serialize_scene(scene: &Scene, path: &str) -> Result<()> {
    let extensions = 
         ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES;
    let config = ron::ser::PrettyConfig::new()
        .struct_names(false)
        .extensions(extensions);
    let data = ron::ser::to_string_pretty(&scene, config)?;
    let mut file = File::create(path)?;
    file.write(data.as_bytes())?;
    return Ok(());
}

pub fn deserialize_scene(path: &str) -> Result<Scene> {
    let data = fs::read_to_string(path)?;
    let world = ron::from_str::<Scene>(&data)?;
    return Ok(world);
}

pub fn get_time_prediction(rays: u32, camera: &Camera, world: &impl Hittable) -> Duration {
    let width = camera.get_config().width;
    let height = camera.get_config().height;
    let samples = 10;

    let mut rng = thread_rng();
    let mut elapsed = Vec::new();
    for _ in 0..samples {
        let w = rng.gen_range(0..width);
        let h = rng.gen_range(0..height);
        let time = Instant::now();
        camera.trace_ray(w, h, world);
        elapsed.push(time.elapsed());
    }
    let sum = elapsed.iter().map(|x| x.as_secs_f64()).sum::<f64>();
    let average = sum / samples as f64;
    dbg!(elapsed);
    dbg!(average);

    // time on a single thread
    let single_time = average * rays as f64;
    let time = single_time / rayon::current_num_threads() as f64;

    println!("estimated time: {}", time);
    return Duration::from_secs_f64(time);
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

pub fn create_ppm_file(file: &str, buf: &Vec<u8>, width: u32, height: u32) -> Result<()> {
    let mut file = File::create(file)?;
    let ppm = format!("P6\n {:?} {:?}\n255\n", width, height);
    file.write(ppm.as_bytes())?;
    file.write(buf.as_slice())?;
    Ok(())
}

pub fn create_image_file(file: &str, buf: &Vec<u8>, width: u32, height: u32) -> Result<()> {
    image::save_buffer(file, buf, width, height, ExtendedColorType::Rgb8)?;
    Ok(())
}

pub fn load_hdri(path: &str) -> Result<Image> {
    let hdri = open(path)?;
    let hdri = hdri.into_rgb8();
    return Ok(Image::from(hdri));
}

pub fn linear_plane_index(len: usize, width: u32, row: u32, column: u32) -> usize {
    len - (row * width + column) as usize
}
