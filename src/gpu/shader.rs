use super::instance::Instance;
use anyhow::Result;
use ash::vk;
use std::{
    ffi::CStr,
    fs::{ File },
    io::Read,
};

pub struct Shader<'a> {
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo<'a>,
}

impl<'a> Shader<'a> {
    pub fn new(device: &Instance, path: &'a str, stage: vk::ShaderStageFlags) -> Result<Self> {
        let mut buf = Vec::<u8>::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut buf)?;
        let ptr = buf.as_ptr() as *const u32;
        let module_info = unsafe {
            vk::ShaderModuleCreateInfo::default()
                .code(std::slice::from_raw_parts(ptr, buf.len() / 4))
        };

        let module = unsafe { device.device.create_shader_module(&module_info, None)? };
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .name(CStr::from_bytes_with_nul(b"main\0")?)
            .module(module)
            .stage(stage);
        return Ok(Self { module, stage_info });
    }

    pub fn destroy(&self, instance: &Instance) {
        unsafe { instance.destroy_shader_module(self.module, None) };
    }
}
