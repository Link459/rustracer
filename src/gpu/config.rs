use ash::vk;

use super::shader::Shader;

pub struct Config<'a> {
    pub recursion_depth: u32,
    pub shaders: &'a [Shader],
    pub groups: &'a [vk::RayTracingShaderGroupCreateInfoKHR],
    pub layout: vk::PipelineLayout,
}
