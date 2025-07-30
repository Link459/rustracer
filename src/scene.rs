use serde::{Deserialize, Serialize};

use crate::{camera::CameraConfig, render::RenderSettings, world::World};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera: CameraConfig,
    pub config:RenderSettings,
    pub world: World,
    pub lights: World,
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
