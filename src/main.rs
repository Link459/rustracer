#![feature(iter_partition_in_place)]
#![allow(dead_code)]
#![allow(clippy::needless_return)]

use anyhow::Result;
use bvh::BvhNode;
use camera::Camera;
use present::Presentation;
use scene::Scene;
use std::{env, time::Instant};

use crate::utils::cmd_seperator;

pub type Float = f32;
pub mod consts {
    use crate::Float;
    pub const PI: Float = 3.14159265358979323846264338327950288;
    pub const TAU: Float = 6.28318530717958647692528676655900577;
    pub const PHI: Float = 1.618033988749894848204586834365638118;
    pub const EGAMMA: Float = 0.577215664901532860606512090082402431;
    pub const FRAC_PI_2: Float = 1.57079632679489661923132169163975144;
    pub const FRAC_PI_3: Float = 1.04719755119659774615421446109316763;
    pub const FRAC_PI_4: Float = 0.785398163397448309615660845819875721;
    pub const FRAC_PI_6: Float = 0.52359877559829887307710723054658381;
    pub const FRAC_PI_8: Float = 0.39269908169872415480783042290993786;
    pub const FRAC_1_PI: Float = 0.318309886183790671537767526745028724;
    pub const FRAC_1_SQRT_PI: Float = 0.564189583547756286948079451560772586;
    pub const FRAC_1_SQRT_2PI: Float = 0.398942280401432677939946059934381868;
    pub const FRAC_2_PI: Float = 0.636619772367581343075535053490057448;
    pub const FRAC_2_SQRT_PI: Float = 1.12837916709551257389615890312154517;
    pub const SQRT_2: Float = 1.41421356237309504880168872420969808;
    pub const FRAC_1_SQRT_2: Float = 0.707106781186547524400844362104849039;
    pub const SQRT_3: Float = 1.732050807568877293527446341505872367;
    pub const FRAC_1_SQRT_3: Float = 0.577350269189625764509148780501957456;
    pub const E: Float = 2.71828182845904523536028747135266250;
}

mod aabb;
mod bvh;
mod camera;
mod hittable;
mod image;
mod integrator;
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
mod sampler;
mod scene;
mod texture;
mod utils;
mod vec3;
mod world;
mod world_options;

struct Settings {}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    /*if args.len() > 3 {
        panic!();
    }*/

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
        mut config,
        mut world,
        lights,
    } = scene;

    cmd_seperator("Scene");
    println!(
        "objects: {}\nlights: {}",
        world.entities.len(),
        lights.entities.len()
    );
    cmd_seperator("Camera");
    println!("{}", camera);
    cmd_seperator("Config");
    println!("{}", config);
    cmd_seperator("BVH");

    world.extend(lights.clone());
    let camera_config = camera;

    println!("generating bvh...");
    let now = Instant::now();
    let world = BvhNode::from_world(world);
    //let world = Bvh::from_world(world);
    println!("time to generate bvh: {:?}", now.elapsed());

    cmd_seperator("Statistics");

    config = utils::parse_render_settings(&args, config);

    let camera = Camera::from_camera_config(camera_config, &config);

    let rays_to_trace = config.width * config.height * config.samples;
    let ray_time = utils::get_time_prediction(rays_to_trace, &camera, &world);

    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");
    println!("estimated time: {}s", ray_time.as_secs());
    cmd_seperator("Rendering");

    let event_loop = present::create_present_loop()?;
    let proxy = event_loop.create_proxy();

    let integrator = integrator::simple_path_integrator::SimplePathIntegrator::new(
        camera.clone(),
        world.clone(),
        lights,
        config.clone(),
    );

    let albedo_integrator = integrator::auxiliary_integrator::AlbedoIntegrator::new(world.clone());
    let normal_integrator = integrator::auxiliary_integrator::NormalIntegrator::new(world);

    let mut app = Presentation::new(config.width, config.height, config.samples as Float);

    let mut render =
        integrator::ImageIntegrator::new(camera.clone(), config.clone(), integrator, proxy.clone());

    let mut albedo = integrator::ImageIntegrator::new(
        camera.clone(),
        config.clone(),
        albedo_integrator,
        proxy.clone(),
    );
    let mut normal = integrator::ImageIntegrator::new(camera, config, normal_integrator, proxy);

    let handle = std::thread::spawn(move || -> Result<()> {
        render.render();

        render.image.clone().save("out.png")?;
        normal.render();
        normal.image.clone().save("normal.png")?;

        albedo.render();
        albedo.image.clone().save("albedo.png")?;

        let start = Instant::now();
        let device = oidn::Device::new();
        oidn::RayTracing::new(&device)
            .srgb(false)
            .hdr(true)
            .image_dimensions(render.image.width as usize, render.image.height as usize)
            .albedo_normal(&albedo.image.buffer, &normal.image.buffer)
            .filter_quality(oidn::Quality::High)
            .filter_in_place(&mut render.image.buffer)
            .expect("Error setting up denoising filter");

        if let Err(e) = device.get_error() {
            eprintln!("Error denoising image: {}", e.1);
        }

        let end = start.elapsed();
        println!("Denoised image in {:?}", end);

        render.image.save("denoised.png")?;
        return Ok(());
    });

    event_loop.run_app(&mut app)?;

    handle.join().unwrap()?;
    return Ok(());
}
