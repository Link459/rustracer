use crate::{
    camera::Camera,
    hittable::Hittable,
    integrator::Integrator,
    interval::Interval,
    light::{LightStore, UniformLightSampler},
    material::MaterialStore,
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
};

pub struct DirectLightingIntegrator<'world, W> {
    camera: Camera,
    world: &'world W,
    lights: UniformLightSampler,
    materials: MaterialStore,
    config: RenderSettings,
}

impl<'world, W> DirectLightingIntegrator<'world, W>
where
    W: Hittable,
{
    pub fn new(
        camera: Camera,
        world: &'world W,
        lights: LightStore,
        materials: MaterialStore,
        config: RenderSettings,
    ) -> Self {
        Self {
            camera,
            world,
            lights: UniformLightSampler::new(lights),
            materials,
            config,
        }
    }

    fn li(&self, mut _ray: Ray, mut _depth: u32) -> Vec3 {
        let l = Vec3::ZERO;

        return l;
    }

    
}

impl<'world, W> Integrator for DirectLightingIntegrator<'world, W>
where
    W: Hittable,
{
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3 {
    fn pixel(&self, ray: &Ray) -> Vec3 {
        return self.li(*ray, 0);
    }

    fn name() -> &'static str {
        return "DirectLightingIntegrator";
    }
}
