use thiserror::Error;

#[derive(Error, Debug)]
pub enum GpuError<'a> {
    #[error("the command buffer is invalid expected to atleast contain {0}")]
    InvalidCommandBuffers(u32),
    #[error("the gpu ran out of memory")]
    HostOutOfMemory,
    #[error("{0}")]
    Message(&'a str),
}
