use anyhow::{self, Result};
use rustracer::{camera::Camera, render::RenderSettings};

use rand::{rngs::SmallRng, SeedableRng};
use rustracer::present::PresentationApp;
use rustracer::scene::Scene;
use std::{env, time::Instant};

use rustracer::{
    bvh::BvhNode,
    denoise::denoise,
    hittable::Hittable,
    integrator::{
        accumulating_integrator::AccumulatingIntegrator, present_integrator::PresentIntegrator,
    },
    integrator::{
        auxiliary_integrator::GBufferIntegrators, AlbedoIntegrator, ImageIntegrator,
        NormalIntegrator, SimplePathIntegrator,
    },
    light::LightStore,
    material::MaterialStore,
    present,
    sampler::{IndependentSampler, Sampler},
    settings::{PresentSettings, Settings},
    utils,
    utils::cmd_seperator,
    world_options, Float,
};

fn create_integrators<'world, W: Hittable + Clone, S: Sampler + Clone + Sync>(
    camera: Camera,
    lights: LightStore,
    materials: MaterialStore,
    sampler: S,
    settings: &Settings,
    use_samples: bool,
    bvh: &'world W,
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
    );

    let albedo_integrator = ImageIntegrator::new(
        camera.clone(),
        AlbedoIntegrator::new(bvh, materials),
        &settings,
        false,
        sampler.clone(),
    );

    let normal_integrator = ImageIntegrator::new(
        camera,
        NormalIntegrator::new(bvh),
        &settings,
        false,
        sampler,
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

    let render_settings = RenderSettings::default();
    let mut settings = Settings::parse(&args, render_settings);
    let scene = match settings.scene_settings {
        rustracer::settings::SceneSettings::Index(idx) => world_options::choose_scene_by_index(idx),
        rustracer::settings::SceneSettings::Path(ref path_buf) => {
            utils::deserialize_scene(path_buf.to_str().expect("Failed to convert Path to str"))?
        }
    };
    /*let scene = if args.len() == 1 {
        utils::deserialize_scene(&args[0])?
    } else {
        //world_options::choose_scene()
        world_options::random_world()
    };*/

    println!("loading scene took: {:?}", now.elapsed());

    let Scene {
        camera,
        config: _render_settings,
        world,
        lights,
        materials,
        skybox,
    } = scene;

    settings.render_settings.skybox = skybox;

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
    //let bvh = rustracer::bvh::builder::BvhBuilder::from_world(world).build();

    //println!("{}", world);
    println!("time to generate bvh: {:?}", now.elapsed());

    cmd_seperator("Statistics");

    let camera = Camera::from_camera_config(camera_config, &settings.render_settings);

    let rays_to_trace = settings.render_settings.width
        * settings.render_settings.height
        * settings.render_settings.samples;
    let ray_time = rustracer::utils::get_time_prediction(rays_to_trace, &camera, &bvh);

    let rays_to_trace = utils::number_with_decimals(rays_to_trace as usize);
    println!("rays to be traced: {rays_to_trace}");
    println!("estimated time: {}s", ray_time.as_secs());
    cmd_seperator("Rendering");

    let mut app = PresentationApp::new(
        settings.render_settings.width,
        settings.render_settings.height,
        settings.render_settings.samples as Float,
    );

    let event_loop = present::create_present_loop()?;
    let proxy = event_loop.create_proxy();
    //let proxy = PresentProxy::new(&app);

    let sampler = IndependentSampler::new(SmallRng::from_rng(&mut rand::rng()));
    let use_samples = match settings.present_settings {
        PresentSettings::OnceDone => true,
        PresentSettings::Accumulate => false,
    };

    let render_thread_handle = match settings.present_settings {
        PresentSettings::OnceDone => std::thread::spawn(move || -> Result<()> {
            let (mut render, mut gbuffer) = create_integrators(
                camera,
                lights,
                materials,
                sampler,
                &settings,
                use_samples,
                &bvh,
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

            let mut accumulator = PresentIntegrator::new(&result, proxy);
            accumulator.render();
            result.save(&settings, "denoised.png")?;
            cmd_seperator("");
            return Ok(());
        }),
        PresentSettings::Accumulate => std::thread::spawn(move || -> Result<()> {
            let (render, _gbuffer) = create_integrators(
                camera,
                lights,
                materials,
                sampler,
                &settings,
                use_samples,
                &bvh,
            );
            let mut accumulator = AccumulatingIntegrator::new(render, proxy);
            accumulator.render();

            return Ok(());
        }),
    };

    event_loop.run_app(&mut app)?;

    render_thread_handle.join().unwrap()?;
    return Ok(());
}
