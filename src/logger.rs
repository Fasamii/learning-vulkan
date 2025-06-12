const LOG_LEVEL: u8 = 3;

use std::ffi::CStr;

use ash::vk::{self, QueueFamilyProperties};

pub enum Logger {
    Empty,
    Error,
    Fix,
    Info,
}

impl Logger {
    pub fn log(status: Logger, msg: &str) {
        match status {
            Logger::Empty if LOG_LEVEL > 3 => println!("\x1b[48;5;44m \x1b[0m - {msg}"),
            Logger::Info if LOG_LEVEL > 2 => println!("\x1b[48;5;5m \x1b[0m - {msg}"),
            Logger::Fix if LOG_LEVEL > 1 => println!("\x1b[48;5;2m \x1b[0m - {msg}"),
            Logger::Error if LOG_LEVEL > 0 => println!("\x1b[48;5;1m \x1b[0m - {msg}"),
            _ => (),
        }
    }
    pub fn space() {
        println!("\x1b[48;5;4m \x1b[0m");
    }
}

pub struct VkLog;

impl VkLog {
    pub fn gpu_name(props: vk::PhysicalDeviceProperties2) {
        unsafe {
            Logger::log(
                Logger::Info,
                &format!(
                    "GPU Name : {:?}",
                    CStr::from_ptr(props.properties.device_name.as_ptr())
                ),
            );
        };
    }

    pub fn memory(props: vk::PhysicalDeviceMemoryProperties2) {
        Logger::log(
            Logger::Info,
            &format!(
                "Memory Types Available : {}",
                props.memory_properties.memory_type_count
            ),
        );

        for i in 0..props.memory_properties.memory_type_count {
            let mem_type = props.memory_properties.memory_types[i as usize];
            let mem_heap = props.memory_properties.memory_heaps[mem_type.heap_index as usize];
            let mut properties: Vec<&str> = Vec::new();

            if mem_type
                .property_flags
                .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            {
                properties.push(&"LOCAL");
            }
            if mem_type
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
            {
                properties.push(&"HOST_VISIVLE");
            }
            if mem_type
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            {
                properties.push(&"HOST_COHERENT");
            }
            if mem_type
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_CACHED)
            {
                properties.push(&"HOST_CACHED");
            }
            if mem_type
                .property_flags
                .contains(vk::MemoryPropertyFlags::LAZILY_ALLOCATED)
            {
                properties.push(&"LAZILY_ALLOCATED");
            }

            Logger::log(
                Logger::Info,
                &format!(
                    "{} > Type : {:^8} : {:>8} MB : FLAGS > {:?}",
                    i,
                    if mem_heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                        "GPU-VRAM"
                    } else {
                        "RAM"
                    },
                    (mem_heap.size / 1024 / 1024),
                    properties,
                ),
            );
        }
    }

    pub fn queue_families(props: Vec<QueueFamilyProperties>) {
        for (index, queue_family) in props.iter().enumerate() {
            let mut capabilities: Vec<&str> = Vec::new();
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                capabilities.push(&"GRAPHICS");
            }
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                capabilities.push(&"COMPUTE");
            }
            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                capabilities.push(&"TRANSFER");
            }
            Logger::log(
                Logger::Info,
                &format!(
                    "Family {} : {} queues > {:?}",
                    index, queue_family.queue_count, capabilities
                ),
            );
        }
    }
}
