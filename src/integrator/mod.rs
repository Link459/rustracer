use crate::{image::Image, sampler::Sampler, scene::Scene};

trait Integrator {
    fn render(&mut self, scene: &Scene);
}

struct Renderer {
    integrator: Box<dyn Integrator>,
    image: Image,
    sampler: Box<dyn Sampler>,
}
