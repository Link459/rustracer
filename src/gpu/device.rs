use std::ops::Deref;

use ash::{
    extensions::ext,
    extensions::{self, khr},
    vk,
};

use anyhow::Result;
pub struct Device {
    pub device: ash::Device,
    pub pdevice: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub queue_index: u32,
}

impl Device {
    pub fn new(
        instance: &ash::Instance,
        entry: &ash::Entry,
        surface: vk::SurfaceKHR,
        surface_loader: &khr::Surface,
    ) -> Result<Self> {
        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };
        let (pdevice, queue_family_index) = pdevices
            .iter()
            .find_map(|pdevice| unsafe {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .expect("Couldn't find suitable device.");

        let queue_family_index = queue_family_index as u32;
        let device_extension_names_raw = [
            khr::Swapchain::name().as_ptr(),
            khr::RayTracingPipeline::name().as_ptr(),
            khr::AccelerationStructure::name().as_ptr(),
            khr::DeferredHostOperations::name().as_ptr(),
            khr::BufferDeviceAddress::name().as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device: ash::Device =
            unsafe { instance.create_device(pdevice, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            device,
            pdevice,
            graphics_queue,
            queue_index: queue_family_index,
        })
    }
}

impl Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
