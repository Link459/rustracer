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
    frame: u32,
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
            frame: 0,
            proxy,
        }
    }

    pub fn render(&mut self) {
        let Self {
            integrator,
            accumulating_image,
            present_image,
            frame,
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
            let original_image = integrator.get_image_ref();
            let accum = accumulating_image.as_mut().unwrap();

            struct PtrWrapper(*const Float);
            unsafe impl Sync for PtrWrapper {}
            unsafe impl Send for PtrWrapper {}
            let buffer_ptr = PtrWrapper(accum.buffer.as_ptr());
            let buffer_ref = &buffer_ptr;

            /*accum.compute_parallel(|w, h| {
                let index = original_image.index(h, w);
                let color = original_image.read(index);
                let other = Image::read_ptr(buffer_ref.0, index);
                return other + color;
            });

            let samples = *frame as Float;
            present_image.as_mut().unwrap().compute_present(
                |w, h| {
                    let index = accum.index(h, w);
                    let color = accum.read(index);
                    return color / samples;
                },
                proxy,
            );*/

            accum.compute_present(
                |w, h| {
                    let index = original_image.index(h, w);
                    let color = original_image.read(index);
                    let prev_color = Image::read_ptr(buffer_ref.0, index);
                    let weight = 1.0 / (*frame + 1) as Float;

                    let accumulated = prev_color * (1.0 - weight) + color * weight;

                    return accumulated;
                },
                proxy,
            );

            *frame += 1;
            if *frame % 10 == 0 {
                println!("current samples: {}", frame)
            }
        }
    }
}
