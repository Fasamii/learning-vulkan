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
        })
    }

    fn destroy(self) {
        unsafe { self.device.destroy_device(None) };
        unsafe { self.instance.destroy_instance(None) };
    }
}

fn main() {
    let app = match GraphicsApp::new() {
        Ok(app) => app,
        Err(err) => {
            panic!("[!] Failed to initialize vulkan :: {err}")
        }
    };

    app.destroy();
}
