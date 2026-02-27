use crate::{image::Image, integrator::ImageIntegrator, present::PresentProxy, Float};

pub struct PresentIntegrator<'img> {
    image: &'img Image,
    proxy: PresentProxy,
}

impl<'img> PresentIntegrator<'img> {
    pub fn new(image: &'img Image, proxy: PresentProxy) -> Self {
        Self { image, proxy }
    }

    pub fn render(&mut self) {
        let Self { image, proxy } = self;

        image.present(proxy);
    }
}
