use anyhow::Result;
use image::{open, ExtendedColorType};
use std::{fs::File, io::Write};

use crate::image::Image;

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
