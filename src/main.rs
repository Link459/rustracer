#![feature(iter_partition_in_place)]
#![allow(dead_code)]
#![allow(clippy::needless_return)]

use anyhow::Result;
use bvh::BvhNode;
use camera::Camera;
use present::Presentation;
use scene::Scene;
use std::{env, time::Instant};


mod aabb;
mod bvh;
mod camera;
mod hittable;
mod image;
mod interval;
mod material;
mod model;
mod moving_sphere;
mod onb;
mod pdf;
mod perlin;
mod present;
mod ray;
mod render;
mod scene;
mod texture;
mod utils;
mod vec3;
mod volume;
mod world;
mod world_options;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() > 3 {
        panic!();
    }

    if args.len() == 2 && args[0] == "--save" {
        let scene = world_options::choose_scene();

        utils::serialize_scene(&scene, &args[1])?;
        return Ok(());
    }

    println!("loading scene...");
    let now = Instant::now();
    let scene = if args.len() == 1 {
        utils::deserialize_scene(&args[0])?
    } else {
        world_options::choose_scene()
    };

    println!("loading scene took: {:?}", now.elapsed());

    let Scene {
        camera,
        mut world,
        lights,
    } = scene;

    println!(
        "objects: {}\nlights: {}",
        world.entities.len(),
        lights.entities.len()
    );
    println!("camera:\n{}", camera);

    world.extend(lights.clone());
    let camera_config = camera;

    println!("generating bvh...");
    let now = Instant::now();
    let world = BvhNode::from_world(world);
    //let world = Bvh::from_world(world);
    //println!("{}", world);
    println!("{:?}", world);
    println!("time to generate bvh: {:?}", now.elapsed());

    let camera = Camera::from_camera_config(camera_config);
    let config = camera.get_config().clone();
    let rays_to_trace = config.width * config.height;
    let ray_time = utils::get_time_prediction(rays_to_trace, &camera, &world);
    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");
    println!("estimated time: {}s", ray_time.as_secs());

    println!("starting...");
    let event_loop = present::create_present_loop()?;
    let proxy = event_loop.create_proxy();

    let handle = std::thread::spawn(move || -> Result<()> {
        let image = camera.render(world, lights, proxy)?;

        utils::create_ppm_file("out.ppm", &image.buffer, image.width, image.height)?;

        return Ok(());
    });

    let mut app = Presentation::new(config.width, config.height, config.samples as f64);
    event_loop.run_app(&mut app)?;

    handle.join().unwrap()?;
    return Ok(());
}
