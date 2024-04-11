use anyhow::Result;
use nalgebra_glm::Mat4x4;
use std::mem::size_of;

use ash::vk;

use crate::{gpu::util, vec3::Vec3};

use super::{buffer::Buffer, device::Device, error::GpuError, instance};

#[derive(Clone, Copy)]
pub struct Vertex {
    pos: Vec3,
    color: Vec3,
}

pub struct GpuModel {
    vertices: Vec<Vertex>,
    vertex_buffer: Buffer<Vertex>,
    indices: Vec<u32>,
    index_buffer: Buffer<u32>,
    transform_matrix: Mat4x4,
}

impl GpuModel {
    pub fn new(path: &str, instance: &ash::Instance, device: &Device) -> Result<Self> {
        let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for model in &models {
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let vertex = Vertex {
                    pos: Vec3::new(
                        model.mesh.positions[pos_offset].into(),
                        model.mesh.positions[pos_offset + 1].into(),
                        model.mesh.positions[pos_offset + 2].into(),
                    ),
                    color: Vec3::new(1.0, 1.0, 1.0),
                };

                vertices.push(vertex);
                indices.push(indices.len() as u32);
            }
        }

        let vertex_buffer = Buffer::new(
            instance,
            device,
            0,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::BufferCreateFlags::default(),
        )?;
        vertex_buffer.map(&device, vertices.as_slice())?;
        let index_buffer = Buffer::new(
            instance,
            device,
            0,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::BufferCreateFlags::default(),
        )?;
        index_buffer.map(&device, indices.as_slice())?;

        Ok(Self {
            vertices,
            vertex_buffer,
            indices,
            index_buffer,
            transform_matrix: Mat4x4::zeros(),
        })
    }

    pub fn to_geometry(
        &self,
        device: &Device,
    ) -> (
        vk::AccelerationStructureGeometryKHR,
        vk::AccelerationStructureBuildRangeInfoKHR,
    ) {
        let vertex_address = util::get_buffer_device_address(device, &self.vertex_buffer);
        let index_address = util::get_buffer_device_address(device, &self.index_buffer);

        let vertex_data = vk::DeviceOrHostAddressConstKHR {
            device_address: vertex_address,
        };
        let index_data = vk::DeviceOrHostAddressConstKHR {
            device_address: index_address,
        };

        let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR {
            vertex_format: vk::Format::R32G32B32_SFLOAT,
            vertex_data,
            vertex_stride: size_of::<Vertex>() as u64,

            index_type: vk::IndexType::UINT32,
            index_data,

            max_vertex: self.vertices.len() as u32,

            ..Default::default()
        };

        let geom_data = vk::AccelerationStructureGeometryDataKHR { triangles };

        let geometry = vk::AccelerationStructureGeometryKHR {
            geometry_type: vk::GeometryTypeKHR::TRIANGLES,
            geometry: geom_data,
            flags: vk::GeometryFlagsKHR::OPAQUE,
            ..Default::default()
        };
        let max_primitive_count = self.indices.len() / 3;
        let offset = vk::AccelerationStructureBuildRangeInfoKHR {
            first_vertex: 0,
            primitive_count: max_primitive_count as u32,
            primitive_offset: 0,
            transform_offset: 0,
        };

        return (geometry, offset);
    }

    pub fn to_transform_matrix(&self) -> vk::TransformMatrixKHR {
        let matrix = self
            .transform_matrix
            .as_slice()
            .to_vec()
            .drain(12..16)
            .collect::<Vec<_>>();

        vk::TransformMatrixKHR {
            matrix: matrix.as_slice().try_into().unwrap(),
        }
    }
}
