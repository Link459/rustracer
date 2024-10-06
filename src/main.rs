#![allow(dead_code)]
use crate::bvh::BvhNode;
use ::image::EncodableLayout;
use anyhow::Result;
use present::present;
use std::{
    fs::File,
    io::{stdin, stdout, Write},
};

mod aabb;
mod bvh;
mod camera;
mod hittable;
mod image;
mod interval;
mod material;
mod model;
mod moving_sphere;
mod perlin;
mod present;
mod ray;
mod render;
mod texture;
mod vec3;
mod volume;
mod world;
mod world_options;

macro_rules! option_pair {
    ($name:tt,$fn:expr) => {
        (
            $name,
            $fn as fn() -> (crate::world::World, crate::camera::Camera),
        )
    };
}

fn main() -> Result<()> {
    let options = vec![
        option_pair!("random_world", world_options::random_world),
        option_pair!("random_world_moving", world_options::random_world_moving),
        option_pair!("two_chess_spheres", world_options::two_chess_spheres),
        option_pair!("two_perlin_spheres", world_options::two_perlin_spheres),
        option_pair!("earth", world_options::earth),
        option_pair!("quads", world_options::quads),
        option_pair!("simple_light", world_options::simple_light),
        option_pair!("cornell_box", world_options::cornell_box),
        option_pair!("cornell_smoke", world_options::cornell_smoke),
    ];

    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {}", opt.0);
    }

    print!("choose a scene to render: ");
    stdout().flush()?;
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read line");
    let choice = buf.trim().parse::<usize>().unwrap();
    println!();
    println!("choose scene: {choice}");

    let (mut world, camera) = options[choice].1();

    /*let (mut world, camera) = match choice {
        0 => world_options::random_world(),
        1 => world_options::random_world_moving(),
        2 => world_options::two_chess_spheres(),
        3 => world_options::two_perlin_spheres(),
        4 => world_options::earth(),
        5 => world_options::quads(),
        6 => world_options::simple_light(),
        7 => world_options::cornell_box(),
        8 => world_options::cornell_smoke(),
        _ => panic!("{buf} was not a viable otption"),
    };*/

    let len = world.entities.len();
    //let world = BvhNode::new(&mut world.entities, 0, len);

    let image = camera.render(world)?;

    let mut file = File::create("out.ppm")?;
    let ppm = format!(
        "P6\n {:?} {:?}\n255\n",
        camera.get_config().width,
        camera.get_config().height
    );
    file.write(ppm.as_bytes())?;
    file.write(image.buffer.as_bytes())?;

    present(image)?;
    return Ok(());
}
