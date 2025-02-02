#![allow(dead_code)]
#![allow(clippy::needless_return)]

use anyhow::Result;
use bvh::BvhNode;
use camera::Camera;
use present::Presentation;
use scene::Scene;
use std::{env, time::Instant};
use utils::serialize_scene;

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
        let (world, camera_config) = world_options::choose_scene();
        let scene = Scene {
            camera_config,
            world,
        };
        serialize_scene(&scene, &args[1])?;
        return Ok(());
    }

    let scene = if args.len() == 1 {
        utils::deserialize_scene(&args[0])?
    } else {
        let (world, camera_config) = world_options::choose_scene();
        Scene {
            camera_config,
            world,
        }
    };

    let Scene {
        camera_config,
        world,
    } = scene;

    let now = Instant::now();
    let world = BvhNode::from_world(world);
    println!("time to generate bvh: {:?}", now.elapsed());

    let camera = Camera::from_camera_config(camera_config);
    let config = camera.get_config().clone();
    let rays_to_trace = config.width * config.height ;
    let ray_time = utils::get_time_prediction(rays_to_trace, &camera, &world);
    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");
    println!("estimated time: {}s", ray_time.as_secs());

    let event_loop = present::create_present_loop()?;
    let proxy = event_loop.create_proxy();

    let handle = std::thread::spawn(move || -> Result<()> {
        let image = camera.render(world, proxy)?;

        utils::create_ppm_file("out.ppm", &image.buffer, image.width, image.height)?;

        return Ok(());
    });

    let mut app = Presentation::new(config.width, config.height, config.samples as f64);
    event_loop.run_app(&mut app)?;

    handle.join().unwrap()?;
    return Ok(());
}
