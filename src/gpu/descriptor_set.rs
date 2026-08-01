use ash::{
    prelude::VkResult,
    vk::{self},
};

use crate::gpu::instance;

pub struct DescriptorSet {
    pub layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    storage_image_count: usize,
}

#[repr(C)]
pub struct DescriptorHandle(u32);

impl DescriptorHandle {
    pub const INVALID: u32 = u32::MAX;
}

const STORAGE_IMAGE_BINDING: u32 = 2;

impl DescriptorSet {
    pub fn new(instance: &instance::Instance) -> VkResult<Self> {
        let flags = vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
            | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING;

        let storage_image_count = 10000;
        let storage_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(STORAGE_IMAGE_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(storage_image_count)
            .stage_flags(vk::ShaderStageFlags::ALL);

        /*let sampled_image_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(1)
        .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
        .descriptor_count(storage_image_count)
        .stage_flags(vk::ShaderStageFlags::ALL);*/

        let bindings = [storage_binding];
        let flags = [flags];
        let mut extended_flags =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .push_next(&mut extended_flags)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);

        let layout = unsafe { instance.create_descriptor_set_layout(&layout_info, None)? };

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(storage_image_count)];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let pool = unsafe { instance.create_descriptor_pool(&pool_info, None)? };

        let layouts = [layout];
        let set_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        let set = unsafe { instance.allocate_descriptor_sets(&set_info)?[0] };

        return Ok(Self {
            layout,
            pool,
            set,
            storage_image_count: 0,
        });
    }

    pub fn bind(&mut self, instance: &instance::Instance, view: vk::ImageView) -> DescriptorHandle {
        let new_id = self.storage_image_count;
        self.storage_image_count += 1;

        let image_info = [vk::DescriptorImageInfo::default()
            .image_view(view)
            .image_layout(vk::ImageLayout::GENERAL)];
        let write = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(STORAGE_IMAGE_BINDING)
            .dst_array_element(new_id as u32)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&image_info)];

        unsafe {
            instance.update_descriptor_sets(&write, &[]);
        }

        return DescriptorHandle(0);
    }

    pub fn destroy(&mut self, instance: &instance::Instance) {
        unsafe {
            instance.destroy_descriptor_pool(self.pool, None);
            instance.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
