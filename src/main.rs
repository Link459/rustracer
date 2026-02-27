#![allow(dead_code)]
#![allow(clippy::needless_return)]

use anyhow::Result;
use camera::Camera;
use present::Presentation;
use rand::{rngs::SmallRng, SeedableRng};
use scene::Scene;
use std::{env, time::Instant};
use winit::event_loop::EventLoopProxy;

use crate::{
    bvh::BvhNode,
    denoise::denoise,
    hittable::Hittable,
    integrator::{
        auxiliary_integrator::GBufferIntegrators, AlbedoIntegrator, ImageIntegrator,
        NormalIntegrator, SimplePathIntegrator,
    },
    light::{LightStorage, LightStore},
    material::MaterialStore,
    present::PresentationEvent,
    sampler::{IndependentSampler, Sampler},
    settings::Settings,
    utils::cmd_seperator,
};

pub type Float = f32;
pub mod consts {
    use crate::Float;
    pub const PI: Float = 3.14159265358979323846264338327950288;
    pub const INV_PI: Float = 1.0 / PI;
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
mod denoise;
mod hittable;
mod image;
mod integrator;
mod interval;
mod light;
mod material;
mod model;
mod moving_sphere;
mod onb;
mod pdf;
mod perlin;
mod present;
mod random;
mod ray;
mod render;
mod sampler;
mod scene;
mod settings;
mod texture;
mod utils;
mod vec3;
mod world;
mod world_options;

fn create_integrators<'world, W: Hittable + Clone, S: Sampler + Clone + Sync>(
    camera: Camera,
    lights: LightStore,
    materials: MaterialStore,
    sampler: S,
    settings: &Settings,
    use_samples: bool,
    bvh: &'world W,
    proxy: EventLoopProxy<PresentationEvent>,
) -> (
    ImageIntegrator<SimplePathIntegrator<'world, W>, S>,
    GBufferIntegrators<'world, W, S>,
) {
    let integrator = SimplePathIntegrator::new(
        camera.clone(),
        bvh,
        lights,
        materials.clone(),
        settings.render_settings.clone(),
    );
    let render = ImageIntegrator::new(
        camera.clone(),
        integrator,
        &settings,
        use_samples,
        sampler.clone(),
        Some(proxy.clone()),
    );

    let albedo_integrator = ImageIntegrator::new(
        camera.clone(),
        AlbedoIntegrator::new(bvh, materials),
        &settings,
        false,
        sampler.clone(),
        Some(proxy.clone()),
    );

    let normal_integrator = ImageIntegrator::new(
        camera,
        NormalIntegrator::new(bvh),
        &settings,
        false,
        sampler,
        Some(proxy),
    );

    return (
        render,
        GBufferIntegrators {
            normal: normal_integrator,
            albedo: albedo_integrator,
        },
    );
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

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
        config: render_settings,
        world,
        lights,
        materials,
    } = scene;

    let settings = Settings::parse(&args, render_settings);

    cmd_seperator("Scene");
    println!(
        "objects: {}\nlights: {}",
        world.entities.len(),
        lights.len()
    );
    cmd_seperator("Camera");
    println!("{}", camera);
    cmd_seperator("Config");
    println!("{}", settings.render_settings);
    cmd_seperator("BVH");

    //world.extend(lights.clone());
    let camera_config = camera;

    println!("generating bvh...");
    let now = Instant::now();
    let bvh = BvhNode::from_world(world);
    //BUG: New bvh is horrificly slow
    //let bvh = bvh::builder::BvhBuilder::from_world(world).build();

    //println!("{}", world);
    println!("time to generate bvh: {:?}", now.elapsed());

    cmd_seperator("Statistics");

    let camera = Camera::from_camera_config(camera_config, &settings.render_settings);

    let rays_to_trace = settings.render_settings.width
        * settings.render_settings.height
        * settings.render_settings.samples;
    let ray_time = utils::get_time_prediction(rays_to_trace, &camera, &bvh);

    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");
    println!("estimated time: {}s", ray_time.as_secs());
    cmd_seperator("Rendering");

    let event_loop = present::create_present_loop()?;
    let proxy = event_loop.create_proxy();

    let mut app = Presentation::new(
        settings.render_settings.width,
        settings.render_settings.height,
        settings.render_settings.samples as Float,
    );

    let sampler = IndependentSampler::new(SmallRng::from_rng(&mut rand::rng()));
    let use_samples = match settings.present_settings {
        settings::PresentSettings::OnceDone => true,
        settings::PresentSettings::Accumulate => false,
    };

    let render_thread_handle = match settings.present_settings {
        settings::PresentSettings::OnceDone => std::thread::spawn(move || -> Result<()> {
            let (mut render, mut gbuffer) = create_integrators(
                camera,
                lights,
                materials,
                sampler,
                &settings,
                use_samples,
                &bvh,
                proxy,
            );
            gbuffer.normal.render();
            let normal = gbuffer.normal.get_image();
            //normal.clone().save(&settings, "normal.png")?;

            gbuffer.albedo.render();
            let albedo = gbuffer.albedo.get_image();
            //albedo.clone().save(&settings, "albedo.png")?;

            render.render();
            let mut result = render.get_image();
            //result.clone().save(&settings, "out.png")?;

            utils::save_images(&settings, result.clone(), albedo.clone(), normal.clone())?;

            let start = Instant::now();
            denoise(&mut result, &albedo, &normal);

            let end = start.elapsed();
            println!("Denoised image in {:?}", end);
            result.save(&settings, "denoised.png")?;
            cmd_seperator("");
            return Ok(());
        }),
        settings::PresentSettings::Accumulate => std::thread::spawn(move || -> Result<()> {
            let (mut render, _gbuffer) = create_integrators(
                camera,
                lights,
                materials,
                sampler,
                &settings,
                use_samples,
                &bvh,
                proxy,
            );
            let mut accumulator =
                integrator::accumulating_integrator::AccumulatingIntegrator::new(render);
            accumulator.render();

            return Ok(());
        }),
    };

    event_loop.run_app(&mut app)?;

    render_thread_handle.join().unwrap()?;
    return Ok(());
}
