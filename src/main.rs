#![allow(dead_code)]
use crate::{bvh::BvhNode, interval::Interval};
use anyhow::Result;
use core::panic;
use present::present;
use std::{io::stdin, ptr};

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

fn main() -> Result<()> {
    //present()?;
    let options = vec![
        "random_world",
        "two_chess_spheres",
        "two_perlin_spheres",
        "earth",
        "quads",
        "simple_light",
        "cornell_box",
        "cornell_smoke",
    ];

    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {opt}",);
    }

    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read line");

    println!("choose a scene to render");
    let (mut world, camera) = match buf.trim().parse::<u32>().unwrap() {
        0 => world_options::random_world(),
        1 => world_options::two_chess_spheres(),
        2 => world_options::two_perlin_spheres(),
        3 => world_options::earth(),
        4 => world_options::quads(),
        5 => world_options::simple_light(),
        6 => world_options::cornell_box(),
        7 => world_options::cornell_smoke(),
        _ => panic!("{buf} was not a viable otption"),
    };

    let len = world.entities.len();
    //let world = BvhNode::new(&mut world.entities, 0, len);

    //let world = Bvh::new(world, Interval::default())
    //let world = dbg!(world);
    camera.render(world)?;
    return Ok(());
}
