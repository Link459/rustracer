use crate::{ray::Ray, scene::Scene, vec3::Vec3};

trait Integrator {
    fn render(&mut self, scene: &Scene);
}

trait ImageIntegrator {
    fn evaluate_pixel() -> Vec3;
}

trait RayIntegrator {
    fn sample(ray: Ray) -> Vec3;
}
