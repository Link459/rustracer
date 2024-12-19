#![allow(dead_code)]
use anyhow::Result;
use bvh::BvhNode;
use camera::{Camera, CameraConfig};
use present::Presentation;
use scene::Scene;
use std::{
    env,
    io::{stdin, stdout, Write},
    time::Instant,
};
use utils::serialize_scene;
use world::World;

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

macro_rules! option_pair {
    ($name:tt,$fn:expr) => {
        (
            $name,
            $fn as fn() -> (crate::world::World, crate::camera::CameraConfig),
        )
    };
}

pub fn choose_scene() -> (World, CameraConfig) {
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
        option_pair!("final_world", world_options::final_world),
    ];

    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {}", opt.0);
    }

    print!("choose a scene to render: ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read line");
    let choice = buf.trim().parse::<usize>().unwrap();
    println!();
    println!("choose scene: {}", options[choice].0);

    let (world, camera) = options[choice].1();
    return (world, camera);
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() > 3 {
        panic!();
    }

    if args.len() == 2 {
        if args[0] == "--save" {
            let (world, camera_config) = choose_scene();
            let scene = Scene {
                camera_config,
                world,
            };
            serialize_scene(&scene, &args[1])?;
            return Ok(());
        }
        //scene = utils::deserialize_scene(&args[0])?;
    }

    let scene;
    if args.len() == 1 {
        scene = utils::deserialize_scene(&args[0])?;
    } else {
        let (world, camera_config) = choose_scene();
        scene = Scene {
            camera_config,
            world,
        };
    }

    let Scene {
        camera_config,
        world,
    } = scene;

    let now = Instant::now();
    let world = BvhNode::from_world(world);
    println!("time to generate bvh: {:?}", now.elapsed());

    let camera = Camera::from_camera_config(camera_config);
    let config = camera.get_config().clone();
    let rays_to_trace = config.width * config.height * config.samples;
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
