use serde::{Deserialize, Serialize};

use crate::{
    camera::CameraConfig,
    light::LightStore,
    material::MaterialStore,
    render::{RenderSettings, Skybox},
    world::World,
};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera: CameraConfig,
    pub config: RenderSettings,
    pub world: World,
    pub lights: LightStore, //World,
    pub materials: MaterialStore,
    pub skybox: Skybox,
}

impl From<(World, CameraConfig)> for Scene {
    fn from(value: (World, CameraConfig)) -> Self {
        return Self {
            camera: value.1,
            world: value.0,
            ..Default::default()
        };
    }
}
