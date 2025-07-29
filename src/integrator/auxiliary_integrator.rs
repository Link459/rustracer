use crate::{
    hittable::Hittable, image::Image, integrator::Integrator, interval::Interval,
    material::Material, ray::Ray, vec3::Vec3, Float,
};

pub struct NormalIntegrator<W> {
    world: W,
}

impl<W> NormalIntegrator<W> {
    pub fn new(world: W) -> Self {
        Self { world }
    }
}

impl<W> Integrator for NormalIntegrator<W>
where
    W: Hittable,
{
    fn pixel(&self, ray: &Ray) -> crate::vec3::Vec3 {
        let Some((payload, _material)) = self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return Vec3::ZERO;
        };

        return payload.p;
    }

    fn name() -> &'static str {
        return "NormalIntegrator";
    }
}

pub struct AlbedoIntegrator<W> {
    world: W,
}

impl<W> AlbedoIntegrator<W> {
    pub fn new(world: W) -> Self {
        Self { world }
    }
}

impl<W> Integrator for AlbedoIntegrator<W>
where
    W: Hittable,
{
    fn pixel(&self, ray: &Ray) -> crate::vec3::Vec3 {
        let Some((payload, material)) = self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return Vec3::ZERO;
        };

        let color_from_emit = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

        //let Some(scatter_payload) = material.scatter(ray, &payload) else {
        let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
            return color_from_emit;
        };

        return scatter_payload.attenuation;
    }

    fn name() -> &'static str {
        return "AlbedoIntegrator";
    }
}
