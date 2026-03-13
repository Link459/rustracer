use crate::{
    camera::Camera,
    hittable::Hittable,
    integrator::Integrator,
    interval::Interval,
    material::{lambertian::random_unit_sphere, MaterialStore},
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
    Float,
};

pub struct RandomWalkIntegrator<'world, W> {
    camera: Camera,
    world: &'world W,
    materials: MaterialStore,
    config: RenderSettings,
}

impl<'world, W> RandomWalkIntegrator<'world, W>
where
    W: Hittable,
{
    pub fn new(
        camera: Camera,
        world: &'world W,
        materials: MaterialStore,
        config: RenderSettings,
    ) -> Self {
        Self {
            camera,
            world,
            materials,
            config,
        }
    }

    fn li(&self, mut ray: Ray, depth: u32) -> Vec3 {
        let Some((payload, material_id)) =
            self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
        else {
            return self.config.skybox.call(&ray);
        };

        let material = self.materials.get(material_id);

        let wi = -ray.dir;

        let emitted = material.emitted(&ray.dir, &payload, payload.u, payload.v, &payload.p);

        if depth == self.config.max_depth {
            return emitted;
        }

        let wo = random_unit_sphere();

        ray = Ray::new(payload.p, wo, ray.time);

        let f = material.f(wi, wo);
        let fcos = f * wo.dot(&payload.normal).abs();
        return emitted + fcos * self.li(ray, depth + 1) / (1.0 / (4.0 * crate::consts::PI));
    }
}

impl<'world, W> Integrator for RandomWalkIntegrator<'world, W>
where
    W: Hittable,
{
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3 {
    fn pixel(&self, ray: &Ray) -> Vec3 {
        return self.li(*ray, 0);
    }

    fn name() -> &'static str {
        return "RandomWalkIntegrator";
    }
}
