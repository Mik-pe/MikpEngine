use super::CORE_LOADER;

use erupt::{
    cstr,
    extensions::{ext_debug_utils::*, khr_surface::*, khr_swapchain::*},
    utils::{
        allocator::{Allocator, AllocatorCreateInfo},
        surface,
    },
    vk1_0::*,
    DeviceLoader, InstanceLoader,
};
use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
};
use winit::window::Window;

const LAYER_KHRONOS_VALIDATION: *const c_char = cstr!("VK_LAYER_KHRONOS_validation");

struct SwapChainSupportDetails {
    pub surface_caps: SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

struct QueueFamilyIndices {
    pub graphics_idx: Option<u32>,
}
pub struct VulkanCtx {
    instance: InstanceLoader,
    pub device: DeviceLoader,
    pub physical_device: PhysicalDevice,
    pub allocator: Allocator,
    pub surface: SurfaceKHR,
    pub current_extent: Extent2D,
    pub current_surface_format: SurfaceFormatKHR,
    pub swapchain_image_views: Vec<ImageView>,
    pub swapchain: SwapchainKHR,
    pub swapchain_images: Vec<Image>,
    pub command_pool: CommandPool,
    pub command_buffers: Vec<CommandBuffer>,
    pub queue: Queue,
    _messenger: DebugUtilsMessengerEXT,
}

impl SwapChainSupportDetails {
    pub unsafe fn query_swapchain_support(
        instance: &InstanceLoader,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> SwapChainSupportDetails {
        let surface_caps = unsafe {
            instance.get_physical_device_surface_capabilities_khr(physical_device, surface, None)
        }
        .unwrap();
        let surface_formats = instance
            .get_physical_device_surface_formats_khr(physical_device, surface, None)
            .unwrap();

        let present_modes = instance
            .get_physical_device_surface_present_modes_khr(physical_device, surface, None)
            .unwrap();

        SwapChainSupportDetails {
            surface_caps,
            surface_formats,
            present_modes,
        }
    }

    pub fn choose_present_mode(&self) -> PresentModeKHR {
        self.present_modes
            .iter()
            .find(|format| match **format {
                PresentModeKHR::MAILBOX_KHR => true,
                _ => false,
            })
            .cloned()
            .unwrap_or(PresentModeKHR::FIFO_KHR)
    }

    pub fn choose_surface_format(&self) -> Option<SurfaceFormatKHR> {
        if self.surface_formats.is_empty() {
            None
        } else {
            for surface_format in &self.surface_formats {
                if surface_format.format == Format::B8G8R8A8_SRGB
                    && surface_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR_KHR
                {
                    return Some(*surface_format);
                }
            }

            Some(self.surface_formats[0])
        }
    }
}

impl QueueFamilyIndices {
    pub fn find_queue_families(
        instance: &InstanceLoader,
        surface: SurfaceKHR,
        physical_device: PhysicalDevice,
    ) -> Self {
        let mut queue_family_indices = Self { graphics_idx: None };
        unsafe {
            let family_props =
                instance.get_physical_device_queue_family_properties(physical_device, None);
            queue_family_indices.graphics_idx =
                match family_props
                    .into_iter()
                    .enumerate()
                    .position(|(i, properties)| {
                        properties.queue_flags.contains(QueueFlags::GRAPHICS)
                            && instance
                                .get_physical_device_surface_support_khr(
                                    physical_device,
                                    i as u32,
                                    surface,
                                    None,
                                )
                                .unwrap()
                                == true
                    }) {
                    Some(idx) => Some(idx as u32),
                    None => None,
                };
        };

        queue_family_indices
    }
}

impl VulkanCtx {
    fn create_instance(
        with_validation_layers: bool,
        app_name: &CStr,
        engine_name: &CStr,
        window: &Window,
    ) -> InstanceLoader {
        if with_validation_layers && !check_validation_support() {
            panic!("Validation layers requested, but unavailable!");
        }

        let api_version = CORE_LOADER.lock().unwrap().instance_version();
        println!(
            "Mikpe erupt test: - Vulkan {}.{}.{}",
            erupt::version_major(api_version),
            erupt::version_minor(api_version),
            erupt::version_patch(api_version)
        );
        let mut instance_extensions = surface::enumerate_required_extensions(window).unwrap();
        let mut instance_layers = vec![];
        if with_validation_layers {
            instance_extensions.push(EXT_DEBUG_UTILS_EXTENSION_NAME);
            instance_layers.push(LAYER_KHRONOS_VALIDATION);
        }
        let app_info = ApplicationInfoBuilder::new()
            .application_name(app_name)
            .application_version(erupt::make_version(1, 0, 0))
            .engine_name(engine_name)
            .engine_version(erupt::make_version(1, 0, 0))
            .api_version(erupt::make_version(1, 1, 0));

        let create_info = InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&instance_layers);
        let instance = unsafe {
            CORE_LOADER
                .lock()
                .unwrap()
                .create_instance(&create_info, None, None)
        }
        .unwrap();
        let mut instance = InstanceLoader::new(&CORE_LOADER.lock().unwrap(), instance).unwrap();
        instance.load_vk1_0().unwrap();
        instance.load_vk1_1().unwrap();
        instance
    }

    pub fn create_image_view(
        device: &DeviceLoader,
        image: Image,
        format: Format,
        aspect_mask: ImageAspectFlags,
    ) -> ImageView {
        let create_info = ImageViewCreateInfoBuilder::new()
            .image(image)
            .view_type(ImageViewType::_2D)
            .format(format)
            .components(ComponentMapping {
                r: ComponentSwizzle::IDENTITY,
                g: ComponentSwizzle::IDENTITY,
                b: ComponentSwizzle::IDENTITY,
                a: ComponentSwizzle::IDENTITY,
            })
            .subresource_range(unsafe {
                ImageSubresourceRangeBuilder::new()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .discard()
            });
        unsafe { device.create_image_view(&create_info, None, None) }.unwrap()
    }

    pub fn begin_single_time_commands(&self) -> CommandBuffer {
        let create_info = CommandBufferAllocateInfoBuilder::new()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1);
        unsafe {
            let command_buffer: CommandBuffer =
                self.device.allocate_command_buffers(&create_info).unwrap()[0];
            let begin_info = CommandBufferBeginInfoBuilder::new()
                .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();
            command_buffer
        }
    }

    pub fn end_single_time_commands(&self, command_buffer: CommandBuffer) {
        unsafe {
            let command_buffers = vec![command_buffer];
            self.device.end_command_buffer(command_buffer).unwrap();
            let submit_info = SubmitInfoBuilder::new().command_buffers(&command_buffers);
            self.device
                .queue_submit(self.queue, &vec![submit_info], Fence::null())
                .unwrap();
            self.device.queue_wait_idle(self.queue).unwrap();
            self.device
                .free_command_buffers(self.command_pool, &command_buffers);
        }
    }

    //https://vulkan-tutorial.com/Depth_buffering
    pub fn find_supported_format(
        &self,
        candidates: Vec<Format>,
        tiling: ImageTiling,
        features: FormatFeatureFlags,
    ) -> Format {
        let mut format = None;
        for candidate in candidates {
            let format_props = unsafe {
                self.instance.get_physical_device_format_properties(
                    self.physical_device,
                    candidate,
                    None,
                )
            };

            if tiling == ImageTiling::LINEAR
                && (format_props.linear_tiling_features & features) == features
            {
                format = Some(candidate);
                break;
            } else if tiling == ImageTiling::OPTIMAL
                && (format_props.optimal_tiling_features & features) == features
            {
                format = Some(candidate);
                break;
            }
        }

        dbg!(format);
        format.expect("No acceptable format found!")
    }

    pub fn find_depth_format(&self) -> Format {
        let candidates = vec![
            Format::D32_SFLOAT_S8_UINT,
            Format::D32_SFLOAT,
            Format::D24_UNORM_S8_UINT,
        ];
        let tiling = ImageTiling::OPTIMAL;
        let features = FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;
        self.find_supported_format(candidates, tiling, features)
    }

    pub fn init(
        window: &Window,
        with_validation_layers: bool,
        app_name: CString,
        engine_name: CString,
    ) -> Self {
        let mut instance =
            Self::create_instance(with_validation_layers, &app_name, &engine_name, window);
        let _messenger = create_debug_messenger(&mut instance, with_validation_layers);

        let surface = unsafe { surface::create_surface(&mut instance, window, None) }.unwrap();

        let physical_device = unsafe { pick_physical_device(&instance, surface) }.unwrap();

        let queue_family_indices =
            QueueFamilyIndices::find_queue_families(&instance, surface, physical_device);

        let graphics_queue = queue_family_indices.graphics_idx.unwrap();

        let device = create_device(
            &instance,
            physical_device,
            graphics_queue,
            with_validation_layers,
        );

        let queue = unsafe { device.get_device_queue(graphics_queue, 0, None) };
        let allocator =
            Allocator::new(&instance, physical_device, AllocatorCreateInfo::default()).unwrap();

        let surface_info = unsafe {
            SwapChainSupportDetails::query_swapchain_support(&instance, physical_device, surface)
        };

        let current_surface_format = surface_info.choose_surface_format().unwrap();

        let present_mode = surface_info.choose_present_mode();

        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
        let current_extent = surface_info.surface_caps.current_extent;
        let swapchain = create_swapchain(
            &device,
            surface,
            surface_info.surface_caps,
            current_surface_format,
            present_mode,
        );
        let swapchain_images = unsafe { device.get_swapchain_images_khr(swapchain, None) }.unwrap();

        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views
        let swapchain_image_views: Vec<_> = swapchain_images
            .iter()
            .map(|swapchain_image| {
                Self::create_image_view(
                    &device,
                    *swapchain_image,
                    current_surface_format.format,
                    ImageAspectFlags::COLOR,
                )
            })
            .collect();
        // https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers

        // https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Command_buffers
        let create_info = CommandPoolCreateInfoBuilder::new()
            .queue_family_index(graphics_queue)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { device.create_command_pool(&create_info, None, None) }.unwrap();

        let command_buffers = {
            let allocate_info = CommandBufferAllocateInfoBuilder::new()
                .command_pool(command_pool)
                .level(CommandBufferLevel::PRIMARY)
                .command_buffer_count(swapchain_image_views.len() as _);
            unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap()
        };

        let ctx = Self {
            instance,
            allocator,
            device,
            physical_device,
            surface,
            current_extent,
            current_surface_format,
            swapchain,
            swapchain_image_views,
            swapchain_images,
            command_pool,
            command_buffers,
            queue,
            _messenger,
        };
        ctx
    }

    pub fn recreate_swapchain(&mut self) {
        // self.surface_caps = unsafe {
        //     self.instance.get_physical_device_surface_capabilities_khr(
        //         self.physical_device,
        //         self.surface,
        //         None,
        //     )
        // }
        // .unwrap();

        // println!(
        //     "Got extent: {}x{}",
        //     self.surface_caps.current_extent.width, self.surface_caps.current_extent.height,
        // );

        // let mut image_count = self.surface_caps.min_image_count + 1;
        // if self.surface_caps.max_image_count > 0 && image_count > self.surface_caps.max_image_count
        // {
        //     image_count = self.surface_caps.max_image_count;
        // }

        // let create_info = SwapchainCreateInfoKHRBuilder::new()
        //     .surface(self.surface)
        //     .min_image_count(image_count)
        //     .image_format(self.surface_format.format)
        //     .image_color_space(self.surface_format.color_space)
        //     .image_extent(self.surface_caps.current_extent)
        //     .image_array_layers(1)
        //     .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
        //     .image_sharing_mode(SharingMode::EXCLUSIVE)
        //     .pre_transform(self.surface_caps.current_transform)
        //     .composite_alpha(CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
        //     .present_mode(PresentModeKHR::FIFO_KHR)
        //     .clipped(true)
        //     .old_swapchain(self.swapchain);

        // self.swapchain =
        //     unsafe { self.device.create_swapchain_khr(&create_info, None, None) }.unwrap();

        // self.swapchain_images =
        //     unsafe { self.device.get_swapchain_images_khr(self.swapchain, None) }.unwrap();

        // self.swapchain_image_views = self
        //     .swapchain_images
        //     .iter()
        //     .map(|swapchain_image| {
        //         Self::create_image_view(&self.device, *swapchain_image, self.surface_format.format)
        //     })
        //     .collect();
    }

    pub fn pre_destroy(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            for &image_view in &self.swapchain_image_views {
                self.device.destroy_image_view(image_view, None);
            }
            self.device.destroy_swapchain_khr(self.swapchain, None);

            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.surface, None);

            if !self._messenger.is_null() {
                self.instance
                    .destroy_debug_utils_messenger_ext(self._messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

unsafe fn pick_physical_device(
    instance: &InstanceLoader,
    surface: SurfaceKHR,
) -> Option<PhysicalDevice> {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None) }.unwrap();

    let physical_device = physical_devices.into_iter().max_by_key(|physical_device| {
        is_physical_device_suitable(instance, *physical_device, surface)
    });
    if let Some(device) = physical_device {
        let properties = instance.get_physical_device_properties(device, None);
        println!("Picking physical device: {:?}", unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
        });
    }
    physical_device
}

unsafe fn is_physical_device_suitable(
    instance: &InstanceLoader,
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
) -> u32 {
    let properties = instance.get_physical_device_properties(physical_device, None);
    let mut score = 0;

    match properties.device_type {
        PhysicalDeviceType::DISCRETE_GPU => score += 1000,
        PhysicalDeviceType::INTEGRATED_GPU => score += 100,
        PhysicalDeviceType::CPU => score += 10,
        _ => {}
    }

    score += properties.limits.max_image_dimension2_d;
    let swapchain_support =
        SwapChainSupportDetails::query_swapchain_support(instance, physical_device, surface);

    if swapchain_support.surface_formats.is_empty() && swapchain_support.present_modes.is_empty() {
        score = 0;
    }

    score
}

fn create_device(
    instance: &InstanceLoader,
    physical_device: PhysicalDevice,
    graphics_queue: u32,
    with_validation_layers: bool,
) -> DeviceLoader {
    let device_extensions = vec![KHR_SWAPCHAIN_EXTENSION_NAME];
    let mut device_layers = vec![];
    if with_validation_layers {
        device_layers.push(LAYER_KHRONOS_VALIDATION);
    }

    // https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues
    let queue_create_info = vec![DeviceQueueCreateInfoBuilder::new()
        .queue_family_index(graphics_queue)
        .queue_priorities(&[1.0])];
    let features = PhysicalDeviceFeaturesBuilder::new().sampler_anisotropy(true);

    let create_info = DeviceCreateInfoBuilder::new()
        .enabled_extension_names(&device_extensions)
        .enabled_layer_names(&device_layers)
        .queue_create_infos(&queue_create_info)
        .enabled_features(&features);

    let mut device = DeviceLoader::new(
        &instance,
        unsafe { instance.create_device(physical_device, &create_info, None, None) }.unwrap(),
    )
    .unwrap();
    device.load_vk1_0().unwrap();
    device.load_vk1_1().unwrap();
    device
        .load_khr_swapchain()
        .expect("Couldn't load swapchain!");
    device
}
fn create_swapchain(
    device: &DeviceLoader,
    surface: SurfaceKHR,
    surface_caps: SurfaceCapabilitiesKHR,
    format: SurfaceFormatKHR,
    present_mode: PresentModeKHR,
) -> SwapchainKHR {
    // let swaphchain_support = SwapChainSupportDetails::query_swapchain_support(device);

    // VkSurfaceFormatKHR surfaceFormat = chooseSwapSurfaceFormat(swapChainSupport.formats);
    // VkPresentModeKHR presentMode = chooseSwapPresentMode(swapChainSupport.presentModes);
    // VkExtent2D extent = chooseSwapExtent(swapChainSupport.capabilities);

    let mut image_count = surface_caps.min_image_count + 1;
    if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
        image_count = surface_caps.max_image_count;
    }

    let create_info = SwapchainCreateInfoKHRBuilder::new()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(surface_caps.current_extent)
        .image_array_layers(1)
        .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(SharingMode::EXCLUSIVE)
        .pre_transform(surface_caps.current_transform)
        .composite_alpha(CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(SwapchainKHR::null());
    let swapchain = unsafe { device.create_swapchain_khr(&create_info, None, None) }.unwrap();
    swapchain
}

fn create_debug_messenger(
    instance: &mut InstanceLoader,
    with_validation_layers: bool,
) -> DebugUtilsMessengerEXT {
    if with_validation_layers {
        instance.load_ext_debug_utils().unwrap();

        let create_info = DebugUtilsMessengerCreateInfoEXTBuilder::new()
            .message_severity(
                DebugUtilsMessageSeverityFlagsEXT::VERBOSE_EXT
                    | DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                    | DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT,
            )
            .pfn_user_callback(Some(debug_callback));

        unsafe { instance.create_debug_utils_messenger_ext(&create_info, None, None) }.unwrap()
    } else {
        Default::default()
    }
}

unsafe extern "system" fn debug_callback(
    _message_severity: DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> Bool32 {
    println!(
        "{}",
        CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
    );

    FALSE
}

fn check_validation_support() -> bool {
    let mut layer_count = 0u32;
    let commands = Vk10CoreCommands::load(&CORE_LOADER.lock().unwrap()).unwrap();
    unsafe {
        commands.enumerate_instance_layer_properties.unwrap()(&mut layer_count, 0 as _);
        let mut available_layers: Vec<LayerProperties> = Vec::new();
        available_layers.resize(layer_count as usize, LayerProperties::default());
        commands.enumerate_instance_layer_properties.unwrap()(
            &mut layer_count,
            available_layers.as_mut_ptr(),
        );
        let validation_name = std::ffi::CStr::from_ptr(LAYER_KHRONOS_VALIDATION as _);
        for layer in available_layers {
            let layer_name = std::ffi::CStr::from_ptr(layer.layer_name.as_ptr() as _);
            if layer_name == validation_name {
                return true;
            }
        }
    }

    return false;
}
