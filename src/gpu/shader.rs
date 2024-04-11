use anyhow::Result;
use super::device::Device;
use ash::vk;
use std::{ffi::CStr, fs};

pub struct Shader {
    module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo,
}

impl Shader {
    pub fn new(device: &Device, path: &str, stage: vk::ShaderStageFlags) -> Result<Self> {
        let file = fs::read(path)?;
        let code = file.into_iter().map(|x| x as u32).collect::<Vec<u32>>();
        let module_info = vk::ShaderModuleCreateInfo::builder()
            .code(code.as_slice())
            .build();

        let module = unsafe { device.create_shader_module(&module_info, None)? };
        let stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .name(CStr::from_bytes_with_nul(path.as_bytes())?)
            .module(module)
            .stage(stage)
            .build();
        Ok(Self { module, stage_info })
    }
}
