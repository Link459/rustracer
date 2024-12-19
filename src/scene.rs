use serde::{Deserialize, Serialize};

use crate::{camera::CameraConfig, world::World};

#[derive(Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera_config: CameraConfig,
    pub world: World,
}
