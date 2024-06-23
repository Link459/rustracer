use rand::Rng;

use crate::{
    camera::Camera,
    hittable::{RotateY, Translate},
    interval::Interval,
    material::{Dielectric, DiffuseLight, Lambertian, Material, Metal},
    model::model::Model,
    model::quad::Quad,
    model::sphere::Sphere,
    moving_sphere::MovingSphere,
    render::{Background, RenderConfig},
    texture::{ChessTexture, ImageTexture, NoiseTexture, SolidColor},
    vec3::Vec3,
    volume::ConstantMedium,
    world::World,
};

#[inline]
pub fn random_world() -> (World, Camera) {
    let mut rng = rand::thread_rng();
    let origin = Vec3::new(4.0, 0.2, 0.0);
    let mut world = World::default();

    let chess = ChessTexture::new(
        Box::new(SolidColor::new(Vec3::new(0.2, 0.3, 0.1))),
        Box::new(SolidColor::new(Vec3::new(0.9, 0.9, 0.9))),
    );
    world.add(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::new(chess),
    ));
    for a in -11..11 {
        for b in -11..11 {
            let choose_material = rng.gen::<f64>();
            let center = Vec3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );
            if (center - origin).length() > 0.9 {
                if choose_material < 0.8 {
                    let center2 = center + Vec3::new(0.0, rng.gen_range(0.0..0.5), 0.0);
                    // diffuse
                    world.add(Model::MovingSphere(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        1.0,
                        0.2,
                        Lambertian::new(SolidColor::new(Vec3::new(
                            rng.gen::<f64>() * rng.gen::<f64>(),
                            rng.gen::<f64>() * rng.gen::<f64>(),
                            rng.gen::<f64>() * rng.gen::<f64>(),
                        ))),
                    )));
                } else if choose_material < 0.95 {
                    // metal
                    world.add(Sphere::new(
                        center,
                        0.2,
                        Metal::new(
                            Vec3::new(
                                0.5 * (1.0 + rng.gen::<f64>()),
                                0.5 * (1.0 + rng.gen::<f64>()),
                                0.5 * (1.0 + rng.gen::<f64>()),
                            ),
                            0.5 * rng.gen::<f64>(),
                        ),
                    ));
                } else {
                    // glass
                    world.add(Sphere::new(center, 0.2, Dielectric::new(1.5)));
                }
            }
        }
    }
    world.add(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Dielectric::new(1.5),
    ));
    world.add(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        Lambertian::new(SolidColor::new(Vec3::new(0.4, 0.2, 0.1))),
    ));
    world.add(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0),
    ));

    let config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 400, 100, 50);

    return (world, Camera::default_with_config(config));
}

pub fn two_chess_spheres() -> (World, Camera) {
    let mut world = World::default();
    let chess = ChessTexture::new(
        Box::new(SolidColor::new(Vec3::new(0.2, 0.3, 0.1))),
        Box::new(SolidColor::new(Vec3::new(0.9, 0.9, 0.9))),
    );

    world.add(Sphere::new(
        Vec3::new(0.0, -10.0, 0.0),
        10.0,
        Lambertian::new(chess.clone()),
    ));
    world.add(Sphere::new(
        Vec3::new(0.0, 10.0, 0.0),
        10.0,
        Lambertian::new(chess),
    ));

    return (world, Camera::default());
}

#[inline]
pub fn two_perlin_spheres() -> (World, Camera) {
    let mut world = World::default();
    let perlin = NoiseTexture::new(4.0);

    world.add(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::new(perlin.clone()),
    ));
    world.add(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Lambertian::new(perlin),
    ));

    return (world, Camera::default());
}

#[inline]
pub fn earth() -> (World, Camera) {
    let mut world = World::new();
    let earth = ImageTexture::new("earthmap.jpg");
    let earth_surface = Lambertian::new(earth);
    let globe = Sphere::new(Vec3::ZERO, 2.0, earth_surface);
    world.add(globe);

    return (world, Camera::default());
}

#[inline]
pub fn quads() -> (World, Camera) {
    let mut world = World::default();
    let left_red = Material::Lambertian(Lambertian::from(Vec3::new(1.0, 0.2, 0.2)));
    let back_green = Material::Lambertian(Lambertian::from(Vec3::new(0.2, 1.0, 0.2)));
    let right_blue = Material::Lambertian(Lambertian::from(Vec3::new(0.2, 0.2, 1.0)));
    let upper_orange = Material::Lambertian(Lambertian::from(Vec3::new(1.0, 0.5, 0.0)));
    let lower_teal = Material::Lambertian(Lambertian::from(Vec3::new(0.2, 0.8, 0.8)));

    // Quads
    world.add(Quad::new(
        Vec3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green,
    ));
    world.add(Quad::new(
        Vec3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
    ));

    let aspect_ratio = 1.0;

    let vfov = 80.0;
    let lookfrom = Vec3::new(0.0, 0.0, 9.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let focus_dist = 10.0;
    let aperture = 0.0;

    let config = RenderConfig::with_aspect_ratio(1.0, 400, 500, 200);
    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        aspect_ratio,
        aperture,
        focus_dist,
        Interval::new(0.0, 1.0),
        config,
    );

    return (world, camera);
}

#[inline]
pub fn simple_light() -> (World, Camera) {
    let mut world = World::default();
    let pertext = NoiseTexture::new(4.0);
    let difflight = DiffuseLight::new(SolidColor::new(Vec3::new(4.0, 4.0, 4.0)));
    world.add(Sphere::new(
        Vec3::new(0.0, 7.0, 0.0),
        2.0,
        difflight.clone(),
    ));
    world.add(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::new(pertext.clone()),
    ));
    world.add(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Lambertian::new(pertext),
    ));

    world.add(Quad::new(
        Vec3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    ));

    let mut config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 400, 300, 50);
    config.background = Background::Night;
    let cam = Camera::new(
        Vec3::new(26.0, 3.0, 6.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        16.0 / 9.0,
        0.0,
        10.0,
        Interval::new(0.001, 1.0),
        config,
    );

    return (world, cam);
}

#[inline]
fn box_of_quads(a: &Vec3, b: &Vec3, mat: Material) -> Model {
    // Returns the 3D box (six sides) that contains the two opposite vertices a & b.

    let mut sides = World::default();

    // Construct the two opposite vertices with the minimum and maximum coordinates.
    let min = Vec3::new(f64::min(a.x, b.x), f64::min(a.y, b.y), f64::min(a.z, b.z));
    let max = Vec3::new(f64::max(a.x, b.x), f64::max(a.y, b.y), f64::max(a.z, b.z));

    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    sides.add(Quad::new(
        Vec3::new(min.x, min.y, max.z),
        dx,
        dy,
        mat.clone(),
    )); // front
    sides.add(Quad::new(
        Vec3::new(max.x, min.y, max.z),
        -dz,
        dy,
        mat.clone(),
    )); // right
    sides.add(Quad::new(
        Vec3::new(max.x, min.y, min.z),
        -dx,
        dy,
        mat.clone(),
    )); // back
    sides.add(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dz,
        dy,
        mat.clone(),
    )); // left
    sides.add(Quad::new(
        Vec3::new(min.x, max.y, max.z),
        dx,
        -dz,
        mat.clone(),
    )); // top
    sides.add(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dx,
        dz,
        mat.clone(),
    )); // bottom

    return Model::World(sides);
}

#[inline]
pub fn cornell_box() -> (World, Camera) {
    let mut world = World::default();
    let red = Lambertian::new(SolidColor::new(Vec3::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(SolidColor::new(Vec3::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(SolidColor::new(Vec3::new(0.12, 0.45, 0.15)));
    let light = DiffuseLight::new(SolidColor::new(Vec3::new(15.0, 15.0, 15.0)));

    world.add(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    ));

    let box1 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 330.0, 165.0),
        white.clone(),
    );

    let box1 = RotateY::new(Box::new(box1), 15.0);
    let box1 = Translate::new(Box::new(box1), Vec3::new(265.0, 0.0, 295.0));

    world.add(box1);

    let box2 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 165.0, 165.0),
        white,
    );

    let box2 = RotateY::new(Box::new(box2), -18.0);
    let box2 = Translate::new(Box::new(box2), Vec3::new(130.0, 0.0, 65.0));

    world.add(box2);

    let config = RenderConfig::with_aspect_ratio(1.0, 400, 200, 50);
    let cam = Camera::new(
        Vec3::new(278.0, 278.0, -800.0),
        Vec3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        40.0,
        1.0,
        0.0,
        10.0,
        Interval::new(0.0, 1.0),
        config,
    );

    return (world, cam);
}

#[inline]
pub fn cornell_smoke() -> (World, Camera) {
    let mut world = World::default();
    let red = Lambertian::new(SolidColor::new(Vec3::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(SolidColor::new(Vec3::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(SolidColor::new(Vec3::new(0.12, 0.45, 0.15)));
    let light = DiffuseLight::new(SolidColor::new(Vec3::new(15.0, 15.0, 15.0)));

    world.add(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        light,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    ));

    let box1 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 330.0, 165.0),
        white.clone(),
    );

    let box1 = RotateY::new(Box::new(box1), 15.0);
    let box1 = Translate::new(Box::new(box1), Vec3::new(265.0, 0.0, 295.0));

    world.add(ConstantMedium::new(Box::new(box1), 0.01, Vec3::ZERO));

    let box2 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 165.0, 165.0),
        white,
    );

    let box2 = RotateY::new(Box::new(box2), -18.0);
    let box2 = Translate::new(Box::new(box2), Vec3::new(130.0, 0.0, 65.0));

    world.add(ConstantMedium::new(Box::new(box2), 0.01, Vec3::ONE));

    let config = RenderConfig::with_aspect_ratio(1.0, 200, 200, 50);
    let cam = Camera::new(
        Vec3::new(278.0, 278.0, -800.0),
        Vec3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        40.0,
        1.0,
        0.0,
        10.0,
        Interval::new(0.0, 1.0),
        config,
    );

    return (world, cam);
}
