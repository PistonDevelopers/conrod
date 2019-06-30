use std::sync::Arc;
use vulkano::{
    self,
    device::{Device, Queue},
    format::Format,
    image::SwapchainImage,
    instance::{Instance, PhysicalDevice},
    swapchain::{ColorSpace, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError},
};

use vulkano_win::{self, VkSurfaceBuild};
use winit::{self, EventsLoop};

pub struct Window {
    pub surface: Arc<Surface<winit::Window>>,
    pub swapchain: Arc<Swapchain<winit::Window>>,
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub images: Vec<Arc<SwapchainImage<winit::Window>>>,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str, events_loop: &EventsLoop) -> Self {
        let size = winit::dpi::LogicalSize::new(width as f64, height as f64);
        let (width, height): (u32, u32) = size.into();

        let instance: Arc<Instance> = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
        };

        let cloned_instance = instance.clone();

        let physical: PhysicalDevice =
            vulkano::instance::PhysicalDevice::enumerate(&cloned_instance)
                .next()
                .expect("no device available");

        let surface = winit::WindowBuilder::new()
            .with_dimensions(size)
            .with_resizable(false)
            .with_title(title)
            .build_vk_surface(events_loop, instance.clone())
            .unwrap();

        let queue = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = vulkano::device::DeviceExtensions {
                khr_swapchain: true,
                ..vulkano::device::DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue, 0.5)].iter().cloned(),
            )
            .expect("failed to create device")
        };

        let queue = queues.next().unwrap();
        let ((swapchain, images), _surface_dimensions) = {
            let caps = surface
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            let surface_dimensions = caps.current_extent.unwrap_or([width, height]);
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats
                .iter()
                .filter(|&&(fmt, cs)| format_is_srgb(fmt) && cs ==  ColorSpace::SrgbNonLinear)
                .map(|&(fmt, _)| fmt)
                .next()
                .expect("failed to find sRGB format");

            (
                Swapchain::new(
                    device.clone(),
                    surface.clone(),
                    caps.min_image_count,
                    format,
                    surface_dimensions,
                    1,
                    caps.supported_usage_flags,
                    &queue,
                    SurfaceTransform::Identity,
                    alpha,
                    PresentMode::Fifo,
                    true,
                    None,
                )
                .expect("failed to create swapchain"),
                surface_dimensions,
            )
        };

        Self {
            surface,
            swapchain,
            queue,
            device,
            images,
        }
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        let inner_size = self
            .surface
            .window()
            .get_inner_size()
            .unwrap()
            .to_physical(self.surface.window().get_hidpi_factor());
        Some(inner_size.into())
    }

    pub fn handle_resize(&mut self) -> () {
        // Get the new dimensions for the viewport/framebuffers.
        let new_dimensions = self
            .surface
            .capabilities(self.device.physical_device())
            .expect("failed to get surface capabilities")
            .current_extent
            .unwrap();

        let (new_swapchain, new_images) =
            match self.swapchain.recreate_with_dimension(new_dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return self.handle_resize(),
                Err(err) => panic!("Window couldn't be resized! {:?}", err),
            };

        self.swapchain = new_swapchain;
        self.images = new_images;
    }
}

// Implement the `WinitWindow` trait for `WindowRef` to allow for generating compatible conversion
// functions.
impl conrod_winit::WinitWindow for Window {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        winit::Window::get_inner_size(self.surface.window()).map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        winit::Window::get_hidpi_factor(self.surface.window()) as _
    }
}

// Generate the winit <-> conrod type conversion fns.
conrod_winit::conversion_fns!();

pub fn format_is_srgb(format: Format) -> bool {
    use vulkano::format::Format::*;
    match format {
        R8Srgb |
        R8G8Srgb |
        R8G8B8Srgb |
        B8G8R8Srgb |
        R8G8B8A8Srgb |
        B8G8R8A8Srgb |
        A8B8G8R8SrgbPack32 |
        BC1_RGBSrgbBlock |
        BC1_RGBASrgbBlock |
        BC2SrgbBlock |
        BC3SrgbBlock |
        BC7SrgbBlock |
        ETC2_R8G8B8SrgbBlock |
        ETC2_R8G8B8A1SrgbBlock |
        ETC2_R8G8B8A8SrgbBlock |
        ASTC_4x4SrgbBlock |
        ASTC_5x4SrgbBlock |
        ASTC_5x5SrgbBlock |
        ASTC_6x5SrgbBlock |
        ASTC_6x6SrgbBlock |
        ASTC_8x5SrgbBlock |
        ASTC_8x6SrgbBlock |
        ASTC_8x8SrgbBlock |
        ASTC_10x5SrgbBlock |
        ASTC_10x6SrgbBlock |
        ASTC_10x8SrgbBlock |
        ASTC_10x10SrgbBlock |
        ASTC_12x10SrgbBlock |
        ASTC_12x12SrgbBlock => true,
        _ => false,
    }
}
