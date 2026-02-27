use crate::{
    image::Image,
    integrator::{ImageIntegrator, Integrator},
    present::PresentProxy,
    sampler::Sampler,
    Float,
};

pub struct AccumulatingIntegrator<I, S> {
    integrator: ImageIntegrator<I, S>,
    accumulating_image: Option<Image>,
    present_image: Option<Image>,
    current_samples: u32,
    proxy: PresentProxy,
}

impl<I, S> AccumulatingIntegrator<I, S>
where
    I: Integrator + Sync,
    S: Sampler + Sync,
{
    pub fn new(integrator: ImageIntegrator<I, S>, proxy: PresentProxy) -> Self {
        Self {
            integrator,
            accumulating_image: None,
            present_image: None,
            current_samples: 0,
            proxy,
        }
    }

    pub fn render(&mut self) {
        let Self {
            integrator,
            accumulating_image,
            present_image,
            current_samples,
            proxy,
        } = self;

        if accumulating_image.is_none() {
            *accumulating_image = Some(Image::from(&integrator.config));
        }

        if present_image.is_none() {
            *present_image = Some(Image::from(&integrator.config));
        }

        //let proxy = integrator.proxy.clone().unwrap();
        //integrator.proxy = None;

        loop {
            integrator.render();
            let copy_image = integrator.get_image_ref();
            let accum = accumulating_image.as_mut().unwrap();

            struct PtrWrapper(*const Float);
            unsafe impl Sync for PtrWrapper {}
            unsafe impl Send for PtrWrapper {}
            let buffer_ptr = PtrWrapper(accum.buffer.as_ptr());
            let buffer_ref = &buffer_ptr;

            accum.compute_parallel(|w, h| {
                let index = copy_image.index(h, w);
                let color = copy_image.read(index);
                let other = Image::read_ptr(buffer_ref.0, index);
                return other + color;
            });

            let samples = *current_samples as Float;
            present_image.as_mut().unwrap().compute_present(
                |w, h| {
                    let index = accum.index(h, w);
                    let color = accum.read(index);
                    return color / samples;
                },
                proxy,
            );

            *current_samples += 1;
            if *current_samples % 10 == 0 {
                println!("current samples: {}", current_samples)
            }
        }
    }
}
