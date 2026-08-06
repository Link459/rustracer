use anyhow::Result;
use nalgebra_glm::Mat4x4;
use std::mem::size_of;

use ash::vk;

use crate::vec3::Vec3;

use super::{buffer::Buffer, instance::Instance};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pos: Vec3,
}

#[repr(C)]
pub struct GpuMesh {
    triangles: vk::DeviceAddress,
    indices: vk::DeviceAddress,
    index_count: u32,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    vertex_buffer: Buffer<Vertex>,
    indices: Vec<u32>,
    index_buffer: Buffer<u32>,
    transform_matrix: Mat4x4,
}

impl Mesh {
    pub fn new(path: &str, instance: &Instance) -> Result<Self> {
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
                    //color: Vec3::new(1.0, 1.0, 1.0),
                };

                vertices.push(vertex);
                indices.push(indices.len() as u32);
            }

            /*for i in 0..model.mesh.indices.len() / 3 {
                let index = model.mesh.indices[i];
                let pos_offset = (9 * index) as usize;
                let load_vec3 = |base| {
                    Vec3::new(
                        model.mesh.positions[base],
                        model.mesh.positions[base + 1],
                        model.mesh.positions[base + 2],
                    )
                };
                let vertex = Triangle {
                    pos: [
                        load_vec3(pos_offset),
                        load_vec3(pos_offset + 3),
                        load_vec3(pos_offset + 6),
                    ],
                    //color: Vec3::new(1.0, 1.0, 1.0),
                };

                vertices.push(vertex);
                indices.push(indices.len() as u32);
                //indices.push(*index);
            }*/
        }

        let vertex_buffer = Buffer::new(
            instance,
            vertices.len() as u64 * std::mem::size_of::<Vertex>() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::BufferCreateFlags::default(),
        )?;
        vertex_buffer.map(&instance, vertices.as_slice())?;
        instance.name("VertexBuffer", vertex_buffer.get_buffer())?;
        let index_buffer = Buffer::new(
            instance,
            indices.len() as u64 * std::mem::size_of::<u32>() as u64,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::BufferCreateFlags::default(),
        )?;
        instance.name("IndexBuffer", index_buffer.get_buffer())?;
        index_buffer.map(&instance, indices.as_slice())?;
        Ok(Self {
            vertices,
            vertex_buffer,
            indices,
            index_buffer,
            transform_matrix: Mat4x4::zeros(),
        })
    }

    pub fn to_gpu_mesh(&self) -> GpuMesh {
        return GpuMesh {
            triangles: self.vertex_buffer.get_address(),
            indices: self.index_buffer.get_address(),
            index_count: self.vertices.len() as u32,
        };
    }

    pub fn to_geometry(
        &self,
        device: &Instance,
    ) -> (
        vk::AccelerationStructureGeometryKHR<'_>,
        vk::AccelerationStructureBuildRangeInfoKHR,
    ) {
        let vertex_address = self.vertex_buffer.get_address();
        let index_address = self.index_buffer.get_address();

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
