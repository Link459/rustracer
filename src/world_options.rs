use std::io::{stdin, stdout, Write};

use rand::{thread_rng, Rng};

use crate::{
    bvh::BvhNode,
    camera::CameraConfig,
    interval::Interval,
    material::{DefaultMaterial, Dielectric, DiffuseLight, Lambertian, MaterialStorage, Metal},
    model::{
        quad::Quad,
        sphere::Sphere,
        transform::{RotateY, Translate},
        volume::ConstantMedium,
        Model,
    },
    moving_sphere::MovingSphere,
    render::{Background, RenderConfig},
    scene::Scene,
    texture::{ChessTexture, ImageTexture, NoiseTexture, SolidColor, TextureStorage},
    utils::load_hdri,
    vec3::Vec3,
    world::World,
};

#[inline]
pub fn random_world() -> Scene {
    let mut rng = rand::thread_rng();
    let origin = Vec3::new(4.0, 0.2, 0.0);
    let mut world = World::default();

    let chess = ChessTexture::new(
        SolidColor::new(Vec3::new(0.2, 0.3, 0.1)),
        SolidColor::new(Vec3::new(0.9, 0.9, 0.9)),
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
                    // diffuse
                    world.add(Sphere::new(
                        center,
                        0.2,
                        Lambertian::new(SolidColor::new(Vec3::new(
                            rng.gen::<f64>() * rng.gen::<f64>(),
                            rng.gen::<f64>() * rng.gen::<f64>(),
                            rng.gen::<f64>() * rng.gen::<f64>(),
                        ))),
                    ));
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

    let config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 500, 100, 50);

    return (
        world,
        CameraConfig {
            config,
            ..Default::default()
        },
    )
        .into();
}

pub fn random_world_moving() -> Scene {
    let mut rng = rand::thread_rng();
    let origin = Vec3::new(4.0, 0.2, 0.0);
    let mut world = World::default();

    let chess = ChessTexture::new(
        SolidColor::new(Vec3::new(0.2, 0.3, 0.1)),
        SolidColor::new(Vec3::new(0.9, 0.9, 0.9)),
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
                    world.add(MovingSphere::new(
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
                    ));
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

    let config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 300, 50, 50);

    return (
        world,
        CameraConfig {
            config,
            ..Default::default()
        },
    )
        .into();
}

pub fn two_chess_spheres() -> Scene {
    let mut world = World::default();
    let chess = ChessTexture::new(
        SolidColor::new(Vec3::new(0.2, 0.3, 0.1)),
        SolidColor::new(Vec3::new(0.9, 0.9, 0.9)),
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

    return (world, CameraConfig::default()).into();
}

#[inline]
pub fn two_perlin_spheres() -> Scene {
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

    return (world, CameraConfig::default()).into();
}

pub fn overlapping() -> Scene {
    let mut world = World::new();
    let earth = ImageTexture::new("assets/earthmap.jpg");
    let earth_surface = Lambertian::new(earth);
    let globe = Sphere::new(Vec3::ZERO, 2.0, earth_surface);
    world.add(globe);
    let solid = Lambertian::new(SolidColor::new(Vec3::new(0.5, 0.3, 0.1)));
    let sphere = Sphere::new(Vec3::new(0.5, 0.5, 0.5), 2.0, solid);
    world.add(sphere);

    let config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 200, 50, 50);
    return (world, CameraConfig::from_config(config)).into();
}

#[inline]
pub fn earth() -> Scene {
    let mut world = World::new();
    let earth = ImageTexture::new("assets/earthmap.jpg");
    let earth_surface = Lambertian::new(earth);
    let globe = Sphere::new(Vec3::ZERO, 2.0, earth_surface);
    world.add(globe);

    let config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 200, 50, 50);
    return (world, CameraConfig::from_config(config)).into();
}

#[inline]
pub fn quads() -> Scene {
    let mut world = World::default();
    let left_red = MaterialStorage::Lambertian(Lambertian::from(Vec3::new(1.0, 0.2, 0.2)));
    let back_green = MaterialStorage::Lambertian(Lambertian::from(Vec3::new(0.2, 1.0, 0.2)));
    let right_blue = MaterialStorage::Lambertian(Lambertian::from(Vec3::new(0.2, 0.2, 1.0)));
    let upper_orange = MaterialStorage::Lambertian(Lambertian::from(Vec3::new(1.0, 0.5, 0.0)));
    let lower_teal = MaterialStorage::Lambertian(Lambertian::from(Vec3::new(0.2, 0.8, 0.8)));

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
    let camera = CameraConfig {
        lookfrom,
        lookat,
        vup,
        vfov,
        aspect_ratio,
        aperture,
        focus_dist,
        time: Interval::new(0.0, 1.0),
        config,
    };

    return (world, camera).into();
}

#[inline]
pub fn simple_light() -> Scene {
    let mut world = World::default();
    let mut lights = World::default();

    let pertext = NoiseTexture::new(4.0);
    let difflight = DiffuseLight::new(SolidColor::new(Vec3::new(4.0, 4.0, 4.0)));

    lights.add(Sphere::new(
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

    lights.add(Quad::new(
        Vec3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    ));

    let mut config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 400, 300, 50);
    config.background = Background::Night;
    let cam = CameraConfig {
        lookfrom: Vec3::new(26.0, 3.0, 6.0),
        lookat: Vec3::new(0.0, 2.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 20.0,
        aspect_ratio: 16.0 / 9.0,
        aperture: 0.0,
        focus_dist: 10.0,
        time: Interval::new(0.001, 1.0),
        config,
    };

    return Scene {
        camera: cam,
        world,
        lights,
    };
}

pub fn simple_skybox() -> Scene {
    let mut world = World::default();

    let chess = ChessTexture::new(
        SolidColor::new(Vec3::new(0.2, 0.3, 0.1)),
        SolidColor::new(Vec3::new(0.9, 0.9, 0.9)),
    );
    world.add(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::new(chess),
    ));

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

    let hdri = load_hdri("assets/skybox.hdr").unwrap();
    let skybox = Background::Hdri(TextureStorage::Image(ImageTexture::from(hdri)));
    let mut config = RenderConfig::with_aspect_ratio(16.0 / 9.0, 500, 100, 50);
    config.background = skybox;

    return (
        world,
        CameraConfig {
            config,
            ..Default::default()
        },
    )
        .into();
}

#[inline]
fn box_of_quads(a: &Vec3, b: &Vec3, mat: impl Into<MaterialStorage> + Clone) -> Model {
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
pub fn cornell_box() -> Scene {
    let mut world = World::default();
    let mut lights = World::default();
    let red = Lambertian::new(SolidColor::new(Vec3::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(SolidColor::new(Vec3::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(SolidColor::new(Vec3::new(0.12, 0.45, 0.15)));
    let light = DiffuseLight::new(SolidColor::new(Vec3::new(15.0, 15.0, 15.0)));

    world.add(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));

    lights.add(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        DefaultMaterial::new(),
        //light,
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

    let box1 = RotateY::new(box1, 15.0);
    let box1 = Translate::new(box1, Vec3::new(265.0, 0.0, 295.0));

    world.add(box1);

    let box2 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 165.0, 165.0),
        white,
    );

    let box2 = RotateY::new(box2, -18.0);
    let box2 = Translate::new(box2, Vec3::new(130.0, 0.0, 65.0));

    world.add(box2);

    let samples = 10;
    let mut config = RenderConfig::with_aspect_ratio(1.0, 400, samples, 50);
    config.background = Background::Night;
    let cam = CameraConfig {
        lookfrom: Vec3::new(278.0, 278.0, -800.0),
        lookat: Vec3::new(278.0, 278.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 40.0,
        aspect_ratio: 1.0,
        aperture: 0.0,
        focus_dist: 10.0,
        time: Interval::new(0.0, 1.0),
        config,
    };

    return Scene {
        camera: cam,
        world,
        lights,
    };
}

#[inline]
pub fn cornell_smoke() -> Scene {
    let mut world = World::default();
    let mut lights = World::default();
    let red = Lambertian::new(SolidColor::new(Vec3::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(SolidColor::new(Vec3::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(SolidColor::new(Vec3::new(0.12, 0.45, 0.15)));
    let light = DiffuseLight::new(SolidColor::new(Vec3::new(15.0, 15.0, 15.0)));

    world.add(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    lights.add(Quad::new(
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

    let box1 = RotateY::new(box1, 15.0);
    let box1 = Translate::new(box1, Vec3::new(265.0, 0.0, 295.0));

    world.add(ConstantMedium::new(box1, 0.01, Vec3::ZERO));

    let box2 = box_of_quads(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 165.0, 165.0),
        white,
    );

    let box2 = RotateY::new(box2, -18.0);
    let box2 = Translate::new(box2, Vec3::new(130.0, 0.0, 65.0));

    world.add(ConstantMedium::new(box2, 0.01, Vec3::ONE));

    let mut config = RenderConfig::with_aspect_ratio(1.0, 200, 500, 50);
    config.background = Background::Night;
    let cam = CameraConfig {
        lookfrom: Vec3::new(278.0, 278.0, -800.0),
        lookat: Vec3::new(278.0, 278.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 40.0,
        aspect_ratio: 1.0,
        aperture: 0.0,
        focus_dist: 10.0,
        time: Interval::new(0.0, 1.0),
        config,
    };

    return Scene {
        camera: cam,
        world,
        lights,
    };
}

pub fn final_world() -> Scene {
    let mut box_world = World::new();
    let mut lights = World::new();
    let ground = Lambertian::new(SolidColor::new(Vec3::new(0.48, 0.83, 0.53)));
    let box_per_side = 20;
    for i in 0..box_per_side {
        for j in 0..box_per_side {
            let w: f64 = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1: f64 = thread_rng().gen_range(1.0..101.0);
            let z1 = z0 + w;
            box_world.add(box_of_quads(
                &Vec3::new(x0, y0, z0),
                &Vec3::new(x1, y1, z1),
                ground.clone(),
            ));
        }
    }

    let mut world = World::new();
    world.add(BvhNode::from_world(box_world));

    let light = DiffuseLight::new(SolidColor::new(Vec3::new(7.0, 7.0, 7.0)));
    lights.add(Quad::new(
        Vec3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        light,
    ));

    let center1 = Vec3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Lambertian::new(SolidColor::new(Vec3::new(0.7, 0.3, 0.1)));
    world.add(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        50.0,
        sphere_material,
    ));

    world.add(Sphere::new(
        Vec3::new(260.0, 150.0, 45.0),
        50.0,
        Dielectric::new(1.5),
    ));
    world.add(Sphere::new(
        Vec3::new(0.0, 150.0, 145.0),
        50.0,
        Metal::new(Vec3::new(0.8, 0.8, 0.9), 1.0),
    ));

    let boundary = Sphere::new(Vec3::new(360.0, 150.0, 145.0), 70.0, Dielectric::new(1.5));
    world.add(boundary.clone());
    world.add(ConstantMedium::new(boundary, 0.2, Vec3::new(0.2, 0.4, 0.9)));
    let boundary = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 5000.0, Dielectric::new(1.5));
    world.add(ConstantMedium::new(
        boundary,
        0.0001,
        Vec3::new(1.0, 1.0, 1.0),
    ));

    let emat = Lambertian::new(ImageTexture::new("assets/earthmap.jpg"));
    world.add(Sphere::new(Vec3::new(400.0, 200.0, 400.0), 100.0, emat));
    let pertext = NoiseTexture::new(0.2);
    world.add(Sphere::new(
        Vec3::new(220.0, 280.0, 300.0),
        80.0,
        Lambertian::new(pertext),
    ));

    let mut box_world = World::new();
    let white = Lambertian::new(SolidColor::new(Vec3::new(0.73, 0.73, 0.73)));
    for _ in 0..1000 {
        box_world.add(Sphere::new(
            Vec3::random(&mut rand::thread_rng(), 0.0..165.0),
            10.0,
            white.clone(),
        ));
    }

    let rotate = RotateY::new(BvhNode::from_world(box_world), 15.0);
    world.add(Translate::new(rotate, Vec3::new(-100.0, 270.0, 395.0)));

    let mut config = RenderConfig::with_aspect_ratio(1.0, 300, 350, 4);
    config.background = Background::Night;
    let cam = CameraConfig {
        lookfrom: Vec3::new(478.0, 278.0, -600.0),
        lookat: Vec3::new(278.0, 278.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov: 40.0,
        aspect_ratio: 1.0,
        aperture: 0.0,
        focus_dist: 10.0,
        time: Interval::new(0.0, 1.0),
        config,
    };

    return Scene {
        camera: cam,
        world,
        lights,
    };
}

macro_rules! option_pair {
    ($name:tt,$fn:expr) => {
        ($name, $fn as fn() -> crate::scene::Scene)
    };
}

pub fn choose_scene() -> Scene {
    let options = vec![
        option_pair!("random_world", random_world),
        option_pair!("random_world_moving", random_world_moving),
        option_pair!("two_chess_spheres", two_chess_spheres),
        option_pair!("two_perlin_spheres", two_perlin_spheres),
        option_pair!("overlapping", overlapping),
        option_pair!("earth", earth),
        option_pair!("quads", quads),
        option_pair!("simple_light", simple_light),
        option_pair!("simple_skybox", simple_skybox),
        option_pair!("cornell_box", cornell_box),
        option_pair!("cornell_smoke", cornell_smoke),
        option_pair!("final_world", final_world),
    ];

    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {}", opt.0);
    }

    print!("choose a scene to render: ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read line");
    let choice = buf.trim().parse::<usize>().unwrap();
    println!();
    println!("choose scene: {}", options[choice].0);

    let scene = options[choice].1();
    return scene;
}
