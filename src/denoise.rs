use crate::image::Image;

pub fn denoise(res: &mut Image, albedo: &Image, normal: &Image) {
    let device = oidn::Device::new();
    oidn::RayTracing::new(&device)
        .srgb(false)
        .hdr(true)
        .clean_aux(true)
        .image_dimensions(res.width as usize, res.height as usize)
        .albedo_normal(&albedo.buffer, &normal.buffer)
        .filter_quality(oidn::Quality::High)
        .filter_in_place(&mut res.buffer)
        .expect("Error setting up denoising filter");

    if let Err(e) = device.get_error() {
        eprintln!("Error denoising image: {}", e.1);
    }
}
