#![allow(unused)]
#[allow(unused_imports)]
use ash::{Device, Entry, Instance, vk};
use std::{
    ffi::{CStr, CString},
    fmt::Debug,
};

mod logger;
use logger::{Logger, VkLog};

struct GraphicsApp<'a> {
    entry: Entry,
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    device_props: vk::PhysicalDeviceProperties2<'a>,

    buffer_created: bool,
}

impl GraphicsApp<'_> {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Logger::log(Logger::Empty, "Start...");

        let entry = unsafe { Entry::load()? };
        Logger::log(Logger::Empty, "Init : Entry");

        let app_name = CString::new("Vulkan Example").unwrap();
        Logger::log(
            Logger::Empty,
            &format!("Created : CString (app_name) : {:?}", &app_name),
        );

        let engine_name = CString::new("No Engine").unwrap();
        Logger::log(
            Logger::Empty,
            &format!("Created : CString (engine_name) : {:?}", &engine_name),
        );

        let info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(vk::make_api_version(0, 0, 0, 0))
            .engine_version(vk::make_api_version(0, 0, 0, 0))
            .api_version(vk::make_api_version(0, 0, 0, 0));
        Logger::log(Logger::Empty, "Init : App Info");

        let instance_info = vk::InstanceCreateInfo::default().application_info(&info);
        Logger::log(Logger::Empty, "Init : Instance Info");

        let instance = unsafe { entry.create_instance(&instance_info, None)? };
        Logger::log(Logger::Empty, "Init : Instance");

        let physical_device = unsafe { instance.enumerate_physical_devices()? };
        Logger::log(Logger::Empty, "Searching : Physical Devices");

        let physical_device = physical_device[0];
        Logger::log(Logger::Empty, "Init : Physical Device");

        let mut device_props: vk::PhysicalDeviceProperties2 =
            vk::PhysicalDeviceProperties2::default();
        unsafe { instance.get_physical_device_properties2(physical_device, &mut device_props) };
        VkLog::gpu_name(device_props);

        let mut mem_props: vk::PhysicalDeviceMemoryProperties2 =
            vk::PhysicalDeviceMemoryProperties2::default();
        unsafe { instance.get_physical_device_memory_properties2(physical_device, &mut mem_props) };
        Logger::space();
        VkLog::memory(mem_props);

        let queque_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        Logger::log(Logger::Empty, "Get : Queue family properties");

        let queque_family_index = queque_family_properties
            .iter()
            .enumerate()
            .find(|(_, properties)| properties.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .expect("[!] No graphics queue family");
        Logger::log(Logger::Empty, "Init : queue family index");

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queque_family_index)
            .queue_priorities(&[1.0]);
        Logger::log(Logger::Empty, "Init : Queue Info");

        let queue_props =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        Logger::space();
        VkLog::queue_families(queue_props);

        let device_info =
            vk::DeviceCreateInfo::default().queue_create_infos(std::slice::from_ref(&queue_info));
        Logger::log(Logger::Empty, "Get : Device Info");

        let device = unsafe { instance.create_device(physical_device, &device_info, None)? };
        Logger::log(Logger::Empty, "Init : Device");

        let queue = unsafe { device.get_device_queue(queque_family_index, 0) };
        Logger::log(Logger::Empty, "Get : Device Queue");

        Ok(GraphicsApp {
            entry,
            instance,
            device,
            physical_device,
            queue,
            device_props,

            buffer_created: false,
        })
    }

    fn new_buffer(&self, size: u64) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .expect("info failed for buffer")
        };
        let mem_req = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let mem_props = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        let memory_type_index = (0..mem_props.memory_type_count)
            .find(|&index| {
                let memory_type = mem_props.memory_types[index as usize];
                (mem_req.memory_type_bits & (1 << index)) != 0
                    && memory_type
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
            })
            .expect("failed to find suitable memory");
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_req.size)
            .memory_type_index(memory_type_index);
        let buffer_memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allof memory")
        };
        unsafe {
            self.device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind memory")
        }

        let data_to_wire = [1.0f32, 2.0, 4.0, 3.0];
        let data_ptr = unsafe {
            self.device
                .map_memory(buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .expect("failed to make data_ptr")
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                data_to_wire.as_ptr() as *const u8,
                data_ptr as *mut u8,
                std::mem::size_of_val(&data_to_wire),
            );
        }
        // unmap (end)
        unsafe {
            self.device.unmap_memory(buffer_memory);
        }
        (buffer, buffer_memory)
    }

    fn destroy(self, buffer: Option<vk::Buffer>, mem: Option<vk::DeviceMemory>) {
        if self.buffer_created {
            unsafe { self.device.destroy_buffer(buffer.unwrap(), None) }
            unsafe { self.device.free_memory(mem.unwrap(), None) }
        }
        unsafe { self.device.destroy_device(None) };
        unsafe { self.instance.destroy_instance(None) };
    }
}

fn main() {
    let mut app = match GraphicsApp::new() {
        Ok(app) => app,
        Err(err) => {
            panic!("[!] Failed to initialize vulkan :: {err}")
        }
    };

    let (buffer, mem) = app.new_buffer(32);
    app.buffer_created = true;

    app.destroy(Some(buffer), Some(mem));
}
