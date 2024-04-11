use anyhow::Result;
use ash::vk;

use super::{acceleration_structure::AccelStruct, buffer::UnsafeBuffer, device::Device};

pub struct DescriptorSetLayout {
    layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    fn new(device: &Device, bindings: &[vk::DescriptorSetLayoutBinding]) -> Result<Self> {
        let dsl_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
        let layout = unsafe { device.create_descriptor_set_layout(&dsl_info, None)? };

        Ok(Self { layout })
    }
}

#[derive(Default)]
pub struct DescriptorPool {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
}

impl DescriptorPool {
    pub fn new(
        device: &Device,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> Result<Self> {
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(max_sets)
            .pool_sizes(pool_sizes);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_create_info, None)? };
        Ok(Self {
            pool: descriptor_pool,
            ..Default::default()
        })
    }

    pub fn allocate_sets(
        &self,
        device: &Device,
        layout: &DescriptorSetLayout,
        count: u32,
    ) -> Result<Vec<DescriptorSet>> {
        let layouts = (0..count).map(|_| layout.layout).collect::<Vec<_>>();
        let sets_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);
        let sets = unsafe { device.allocate_descriptor_sets(&sets_alloc_info)? };
        let sets = sets
            .into_iter()
            .map(|set| DescriptorSet { set })
            .collect::<Vec<_>>();

        Ok(sets)
    }
}

pub struct DescriptorSet {
    set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn update(&self, device: &Device, writes: &[DescriptorSetWrite]) -> Result<()> {
        let desc_writes = writes
            .iter()
            .map(|w| match w.kind {
                DesciptorSetWriteKind::AccelerationStructure { accel_struct } => {
                    let binding = [accel_struct.accel_struct];
                    let mut write_as = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                        .acceleration_structures(&binding);

                    let mut write = vk::WriteDescriptorSet::builder()
                        .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                        .dst_binding(w.binding)
                        .dst_set(self.set)
                        .push_next(&mut write_as)
                        .build();
                    write.descriptor_count = 1;
                    return write;
                }
                //DesciptorSetWriteKind::UniformBuffer { buffer } => todo!(),
                _ => panic!("invalid something only supports accels atm"),
            })
            .collect::<Vec<_>>();

        unsafe { device.update_descriptor_sets(&desc_writes, &[]) };
        Ok(())
    }
}

pub struct DescriptorSetWrite<'a> {
    binding: u32,
    kind: DesciptorSetWriteKind<'a>,
}

pub enum DesciptorSetWriteKind<'a> {
    AccelerationStructure { accel_struct: &'a AccelStruct },
    UniformBuffer { buffer: &'a UnsafeBuffer },
}
