#![allow(dead_code)]
use anyhow::Result;
use bvh::BvhNode;
use present::present;
use std::{
    fs::File,
    io::{stdin, stdout, Write},
    time::Instant,
};

mod aabb;
mod bvh;
mod camera;
mod hittable;
mod image;
mod interval;
mod material;
mod model;
mod moving_sphere;
mod perlin;
mod present;
mod ray;
mod render;
mod texture;
mod utils;
mod vec3;
mod volume;
mod world;
mod world_options;

macro_rules! option_pair {
    ($name:tt,$fn:expr) => {
        (
            $name,
            $fn as fn() -> (crate::world::World, crate::camera::Camera),
        )
    };
}

fn main() -> Result<()> {
    let options = vec![
        option_pair!("random_world", world_options::random_world),
        option_pair!("random_world_moving", world_options::random_world_moving),
        option_pair!("two_chess_spheres", world_options::two_chess_spheres),
        option_pair!("two_perlin_spheres", world_options::two_perlin_spheres),
        option_pair!("earth", world_options::earth),
        option_pair!("quads", world_options::quads),
        option_pair!("simple_light", world_options::simple_light),
        option_pair!("cornell_box", world_options::cornell_box),
        option_pair!("cornell_smoke", world_options::cornell_smoke),
    ];

    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {}", opt.0);
    }

    print!("choose a scene to render: ");
    stdout().flush()?;
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read line");
    let choice = buf.trim().parse::<usize>().unwrap();
    println!();
    println!("choose scene: {}", options[choice].0);

    let (world, camera) = options[choice].1();

    let config = camera.get_config();
    let rays_to_trace = config.width * config.height * config.samples;
    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");

    let now = Instant::now();
    let world = BvhNode::from_world(world);
    println!("time to generate bvh: {:?}", now.elapsed());

    let image = camera.render(world)?;

    utils::create_ppm_file("out.ppm", &image.buffer, image.width, image.height)?;

    present(image)?;
    return Ok(());
}
