use crate::{
    hittable::Hittable, integrator::Integrator, interval::Interval, material::MaterialStore,
    ray::Ray, vec3::Vec3, Float,
};

pub struct GBufferIntegrators<'world, W, S> {
    pub normal: super::ImageIntegrator<NormalIntegrator<'world, W>, S>,
    pub albedo: super::ImageIntegrator<AlbedoIntegrator<'world, W>, S>,
}

pub struct NormalIntegrator<'world, W> {
    world: &'world W,
}

impl<'world, W> NormalIntegrator<'world, W> {
    pub fn new(world: &'world W) -> Self {
        Self { world }
    }
}

impl<W> Integrator for NormalIntegrator<'_, W>
where
    W: Hittable,
{
    fn pixel(&self, ray: &Ray) -> Vec3 {
        let Some((payload, _material)) = self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return Vec3::ZERO;
        };

        return payload.normal;
    }

    fn name() -> &'static str {
        return "NormalIntegrator";
    }
}

pub struct AlbedoIntegrator<'world, W> {
    world: &'world W,
    materials: MaterialStore,
}

impl<'world, W> AlbedoIntegrator<'world, W> {
    pub fn new(world: &'world W, materials: MaterialStore) -> Self {
        Self { world, materials }
    }
}

impl<W> Integrator for AlbedoIntegrator<'_, W>
where
    W: Hittable,
{
    fn pixel(&self, ray: &Ray) -> Vec3 {
        let Some((payload, material_id)) =
            self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return Vec3::ZERO;
        };

        let material = self.materials.get(material_id);

        let color_from_emit =
            material.emitted(&ray.dir, &payload, payload.u, payload.v, &payload.p);

        //let Some(scatter_payload) = material.scatter(ray, &payload) else {
        let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
            return color_from_emit;
        };

        return scatter_payload.f;
    }

    fn name() -> &'static str {
        return "AlbedoIntegrator";
    }
}
