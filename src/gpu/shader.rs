use super::instance::Instance;
use anyhow::Result;
use ash::vk;
use std::{ffi::CStr, fs};

pub struct Shader<'a> {
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo<'a>,
}

impl<'a> Shader<'a> {
    pub fn new(device: &Instance, path: &'a str, stage: vk::ShaderStageFlags) -> Result<Self> {
        let file = fs::read(path)?;
        let code = file.into_iter().map(|x| x as u32).collect::<Vec<u32>>();
        let module_info = vk::ShaderModuleCreateInfo::default().code(code.as_slice());

        let module = unsafe { device.device.create_shader_module(&module_info, None)? };
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .name(CStr::from_bytes_with_nul(path.as_bytes())?)
            .module(module)
            .stage(stage);
        return Ok(Self { module, stage_info });
    }
}
