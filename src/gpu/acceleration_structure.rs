use super::command_pool::CommandBuffer;
use super::instance::Instance;
use super::model::GpuModel;
use super::{buffer::UnsafeBuffer, command_pool::CommandPool, device::Device};
use anyhow::Result;
use ash::extensions::khr;
use ash::vk::{self, Packed24_8};

#[derive(Copy, Clone, Default)]
pub struct AccelStruct {
    pub accel_struct: vk::AccelerationStructureKHR,
    pub buffer: UnsafeBuffer,
}

impl AccelStruct {
    pub fn new(
        instance: &Instance,
        accel_loader: &khr::AccelerationStructure,
        create_info: &vk::AccelerationStructureCreateInfoKHR,
    ) -> Result<Self> {
        let buffer = unsafe {
            UnsafeBuffer::new(
                instance,
                &instance.device,
                create_info.size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                vk::BufferCreateFlags::default(),
            )?
        };

        let accel_struct =
            unsafe { accel_loader.create_acceleration_structure(create_info, None)? };

        Ok(Self {
            accel_struct,
            buffer,
        })
    }
}

pub struct BLASInput {
    build_geometry: Vec<vk::AccelerationStructureGeometryKHR>,
    build_range: Vec<vk::AccelerationStructureBuildRangeInfoKHR>,
    flags: vk::BuildAccelerationStructureFlagsKHR,
}

pub struct BuildAS<'a> {
    pub build_info: vk::AccelerationStructureBuildGeometryInfoKHR,
    pub build_sizes: vk::AccelerationStructureBuildSizesInfoKHR,
    pub build_ranges: &'a [vk::AccelerationStructureBuildRangeInfoKHR],
    pub accel_struct: AccelStruct,
    cleanup_as: AccelStruct,
}

fn cmd_create_blas(
    instance: &Instance,
    accel_loader: &khr::AccelerationStructure,
    cmd_buf: CommandBuffer,
    indices: &[u32],
    build_as: &mut [BuildAS],
    scratch_address: vk::DeviceAddress,
    query_pool: Option<vk::QueryPool>,
) -> Result<()> {
    if let Some(pool) = query_pool {
        unsafe {
            instance
                .device
                .reset_query_pool(pool, 0, indices.len().try_into()?)
        };
    }
    let mut query_count = 0;

    let accel_structs = indices
        .iter()
        .map(|x| {
            let i = *x as usize;
            let accel_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .size(build_as[i].build_sizes.acceleration_structure_size)
                .build();

            build_as[i].accel_struct =
                AccelStruct::new(instance, accel_loader, &accel_create_info).unwrap();
            build_as[i].build_info.dst_acceleration_structure =
                build_as[i].accel_struct.accel_struct;
            build_as[i].build_info.scratch_data.device_address = scratch_address;
            unsafe {
                accel_loader.cmd_build_acceleration_structures(
                    *cmd_buf,
                    &[build_as[i].build_info],
                    &[build_as[i].build_ranges],
                )
            }
            let mem_barrier = vk::MemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR)
                .dst_access_mask(vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR);
            unsafe {
                instance.device.cmd_pipeline_barrier(
                    *cmd_buf,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                    vk::DependencyFlags::default(),
                    &[*mem_barrier],
                    &[],
                    &[],
                )
            }

            if let Some(pool) = query_pool {
                query_count += 1;
                unsafe {
                    accel_loader.cmd_write_acceleration_structures_properties(
                        *cmd_buf,
                        &[build_as[i].build_info.dst_acceleration_structure],
                        vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR,
                        pool,
                        query_count,
                    )
                }
            }
        })
        .collect::<Vec<()>>();

    Ok(())
}

fn cmd_compact_blas(
    instance: &Instance,
    device: &Device,
    accel_loader: &khr::AccelerationStructure,
    cmd_buf: CommandBuffer,
    indices: Vec<u32>,
    build_as: &mut Vec<BuildAS>,
    query_pool: vk::QueryPool,
) -> Result<()> {
    let mut query_count = 0;
    let mut compact_sizes = indices.clone();
    unsafe {
        device.get_query_pool_results(
            query_pool,
            0,
            indices.len() as u32,
            compact_sizes.as_mut_slice(),
            vk::QueryResultFlags::WAIT,
        )?
    };
    for i in indices.iter() {
        let i = *i as usize;
        build_as[i].cleanup_as = build_as[i].accel_struct;
        query_count += 1;
        build_as[i].build_sizes.acceleration_structure_size = compact_sizes[query_count].into();
        let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .size(build_as[i].build_sizes.acceleration_structure_size);
        build_as[i].accel_struct = AccelStruct::new(instance, accel_loader, &as_create_info)?;

        let copy_info = vk::CopyAccelerationStructureInfoKHR::builder()
            .src(build_as[i].build_info.dst_acceleration_structure)
            .dst(build_as[i].accel_struct.accel_struct)
            .mode(vk::CopyAccelerationStructureModeKHR::COMPACT);

        unsafe { accel_loader.cmd_copy_acceleration_structure(*cmd_buf, &copy_info) };
    }
    Ok(())
}

fn build_input_into<'a>(
    blas_inputs: &'a [BLASInput],
    accel_loader: &khr::AccelerationStructure,
) -> (Vec<BuildAS<'a>>, u64, u64, u64) {
    let blas_size = blas_inputs.len();
    let mut max_scratch_size = 0;
    let mut compaction_size = 0;
    let mut as_total_size = 0;

    let build_as = (0..blas_size)
        .into_iter()
        .map(|i| {
            let build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .flags(blas_inputs[i].flags)
                .geometries(&blas_inputs[i].build_geometry)
                .build();

            let build_ranges = &blas_inputs[i].build_range;

            let max_prim_count = build_ranges
                .iter()
                .map(|x| x.primitive_count)
                .collect::<Vec<_>>();

            let build_sizes = unsafe {
                accel_loader.get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &build_info,
                    max_prim_count.as_slice(),
                )
            };

            as_total_size += build_sizes.acceleration_structure_size;
            max_scratch_size += build_sizes
                .build_scratch_size
                .max(build_sizes.build_scratch_size);

            if build_info
                .flags
                .contains(vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION)
            {
                compaction_size += 1;
            }

            BuildAS {
                build_info,
                build_sizes,
                build_ranges: &build_ranges,
                accel_struct: AccelStruct::default(),
                cleanup_as: AccelStruct::default(),
            }
        })
        .collect::<Vec<BuildAS<'a>>>();

    return (build_as, max_scratch_size, as_total_size, compaction_size);
}

fn create_blas(
    accel_loader: khr::AccelerationStructure,
    buffer: UnsafeBuffer,
    size: u64,
    offset: u64,
) -> Result<vk::AccelerationStructureKHR> {
    let create_info = vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .buffer(buffer.buffer)
        .size(size)
        .offset(offset)
        .build();
    Ok(unsafe { accel_loader.create_acceleration_structure(&create_info, None)? })
}

fn build_blas(
    instance: &Instance,
    device: &Device,
    accel_loader: &khr::AccelerationStructure,
    cmd_buf: CommandBuffer,
    blas_inputs: Vec<BLASInput>,
) -> Result<Vec<AccelStruct>> {
    let (mut blas_builds, max_scratch_size, as_total_size, compaction_size) =
        build_input_into(&blas_inputs, accel_loader);

    let scratch_buffer = unsafe {
        UnsafeBuffer::new(
            instance,
            device,
            max_scratch_size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::BufferCreateFlags::default(),
        )?
    };

    let scratch_buffer_info = vk::BufferDeviceAddressInfo {
        buffer: scratch_buffer.buffer,
        ..Default::default()
    };

    let scratch_buffer_address = unsafe {
        device
            .device
            .get_buffer_device_address(&scratch_buffer_info)
    };

    let mut temp_query_pool = vk::QueryPool::default();
    if compaction_size > 0 {
        let query_pool_create_info = vk::QueryPoolCreateInfo::builder()
            .query_count(blas_builds.len().try_into()?)
            .query_type(vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR)
            .build();

        temp_query_pool = unsafe { device.create_query_pool(&query_pool_create_info, None)? };
    }
    let mut query_pool = None;
    if temp_query_pool != vk::QueryPool::default() {
        query_pool = Some(temp_query_pool)
    }

    let mut indices = Vec::new();
    let mut batch_size = 0;
    let batch_limit = 256_000_000;
    for i in 0..blas_builds.len() {
        indices.push(i);
        batch_size += blas_builds[i].build_sizes.acceleration_structure_size;

        if batch_size >= batch_limit || i == blas_builds.len() - 1 {
            cmd_buf.record_and_submit(
                device,
                vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                vk::Fence::default(),
                device.graphics_queue,
                &[],
                &[],
                &[],
                |_, cmd| {
                    cmd_create_blas(
                        instance,
                        accel_loader,
                        cmd,
                        &indices.iter().map(|x| *x as u32).collect::<Vec<u32>>(),
                        &mut blas_builds,
                        scratch_buffer_address,
                        query_pool,
                    )?;
                    Ok(())
                },
            )?;
            if let Some(pool) = query_pool {
                cmd_buf.record_and_submit(
                    device,
                    vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    vk::Fence::default(),
                    device.graphics_queue,
                    &[],
                    &[],
                    &[],
                    |_, cmd| {
                        cmd_compact_blas(
                            instance,
                            device,
                            accel_loader,
                            cmd,
                            indices.iter().map(|x| *x as u32).collect(),
                            &mut blas_builds,
                            pool,
                        )?;
                        Ok(())
                    },
                )?;
            }
        }
    }
    if let Some(pool) = query_pool {
        unsafe { device.destroy_query_pool(pool, None) };
    }
    unsafe { device.destroy_buffer(*scratch_buffer, None) };

    let blas = blas_builds
        .iter()
        .map(|x| x.accel_struct)
        .collect::<Vec<AccelStruct>>();

    Ok(blas)
}

fn cmd_create_tlas(
    instance: &Instance,
    device: &Device,
    accel_loader: &khr::AccelerationStructure,
    cmd_buf: CommandBuffer,
    instance_count: u32,
    inst_buffer_addr: vk::DeviceAddress,
    flags: vk::BuildAccelerationStructureFlagsKHR,
) -> Result<(AccelStruct, UnsafeBuffer)> {
    let instances_data = vk::AccelerationStructureGeometryInstancesDataKHR::builder().data(
        vk::DeviceOrHostAddressConstKHR {
            device_address: inst_buffer_addr,
        },
    );

    let top_as_geometry = vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::INSTANCES)
        .geometry(vk::AccelerationStructureGeometryDataKHR {
            instances: *instances_data,
        });
    let geometries = [*top_as_geometry];
    let mode = vk::BuildAccelerationStructureModeKHR::BUILD; //TODO: Implement updating aswell
    let build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
        .flags(flags)
        .geometries(&geometries)
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
        .mode(mode);

    let size_info = unsafe {
        accel_loader.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &build_info,
            &[instance_count],
        )
    };
    let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
        .size(size_info.acceleration_structure_size);
    let tlas = AccelStruct::new(instance, accel_loader, &as_create_info)?;

    let scratch_buffer = unsafe {
        UnsafeBuffer::new(
            instance,
            device,
            size_info.build_scratch_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::BufferCreateFlags::default(),
        )?
    };

    let scratch_info = vk::BufferDeviceAddressInfo {
        buffer: *scratch_buffer,
        ..Default::default()
    };
    let scratch_addr = unsafe { device.get_buffer_device_address(&scratch_info) };
    let build_info = build_info
        .dst_acceleration_structure(tlas.accel_struct)
        .scratch_data(vk::DeviceOrHostAddressKHR {
            device_address: scratch_addr,
        });

    let build_offset_info = vk::AccelerationStructureBuildRangeInfoKHR {
        primitive_count: instance_count,
        primitive_offset: 0,
        first_vertex: 0,
        transform_offset: 0,
    };

    let build_info = build_info.build();
    unsafe {
        accel_loader.cmd_build_acceleration_structures(
            *cmd_buf,
            &[build_info],
            &[&[build_offset_info]],
        )
    }

    Ok((tlas, scratch_buffer))
}

fn build_tlas(
    instance: &Instance,
    device: &Device,
    accel_loader: &khr::AccelerationStructure,
    cmd_buf: CommandBuffer,
    instances: &[vk::AccelerationStructureInstanceKHR],
    flags: vk::BuildAccelerationStructureFlagsKHR,
    //update: bool,
) -> Result<AccelStruct> {
    let instance_buffer = unsafe {
        UnsafeBuffer::new(
            instance,
            device,
            instances.len() as u64,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            vk::BufferCreateFlags::default(),
        )?
    };

    let buffer_info = vk::BufferDeviceAddressInfo {
        buffer: *instance_buffer,
        ..Default::default()
    };
    let buffer_addr = unsafe { device.get_buffer_device_address(&buffer_info) };
    let mem_barrier = vk::MemoryBarrier {
        src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
        dst_access_mask: vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
        ..Default::default()
    };
    unsafe {
        device.cmd_pipeline_barrier(
            *cmd_buf,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            vk::DependencyFlags::default(),
            &[mem_barrier],
            &[],
            &[],
        )
    };

    let (tlas, scratch_buffer) = cmd_create_tlas(
        instance,
        device,
        accel_loader,
        cmd_buf,
        instances.len().try_into()?,
        buffer_addr,
        flags,
    )?;

    unsafe { device.destroy_buffer(*instance_buffer, None) };
    unsafe { device.destroy_buffer(*scratch_buffer, None) };
    Ok(tlas)
}

fn get_blas_device_address(
    accel_loader: &khr::AccelerationStructure,
    blas: &[AccelStruct],
    id: usize,
) -> vk::DeviceAddress {
    let addr_info = vk::AccelerationStructureDeviceAddressInfoKHR {
        acceleration_structure: blas[id].accel_struct,
        ..Default::default()
    };

    unsafe { accel_loader.get_acceleration_structure_device_address(&addr_info) }
}

pub struct TLAS {
    blas: Vec<AccelStruct>,
    tlas: AccelStruct,
}

impl TLAS {
    pub fn new(
        instance: &Instance,
        device: &Device,
        accel_loader: &khr::AccelerationStructure,
        command_pool: &CommandPool,
        models: &[GpuModel],
    ) -> Result<Self> {
        let blas_inputs = models
            .iter()
            .map(|x| {
                let (geometry, range) = x.to_geometry(device);
                BLASInput {
                    build_geometry: [geometry].to_vec(),
                    build_range: [range].to_vec(),
                    flags: vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE,
                }
            })
            .collect();

        let blas = build_blas(
            instance,
            device,
            accel_loader,
            command_pool.get_buffers()[0],
            blas_inputs,
        )?;
        let instances = models
            .iter()
            .map(|x| vk::AccelerationStructureInstanceKHR {
                transform: x.to_transform_matrix(),
                instance_custom_index_and_mask: Packed24_8::new(0, 0xFF),
                instance_shader_binding_table_record_offset_and_flags: Packed24_8::new(
                    0,
                    vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
                ),
                acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                    device_handle: get_blas_device_address(accel_loader, blas.as_slice(), 0),
                },
            })
            .collect::<Vec<vk::AccelerationStructureInstanceKHR>>();

        let tlas = build_tlas(
            instance,
            device,
            accel_loader,
            command_pool.get_buffers()[0],
            instances.as_slice(),
            vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE,
        )?;

        Ok(Self { blas, tlas })
    }
}
