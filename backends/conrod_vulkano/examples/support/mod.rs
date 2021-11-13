use std::sync::Arc;
use vulkano::{
    self,
    device::{Device, Queue},
    image::SwapchainImage,
    instance::Instance,
    swapchain::{ColorSpace, Surface, Swapchain, SwapchainCreationError},
    Version,
};

use vulkano::device::physical::PhysicalDevice;
use vulkano::format::NumericType;
use vulkano::image::ImageUsage;
use vulkano_win::{self, VkSurfaceBuild};

pub struct Window {
    pub surface: Arc<Surface<winit::window::Window>>,
    pub swapchain: Arc<Swapchain<winit::window::Window>>,
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
}

impl Window {
    pub fn new<T>(
        width: u32,
        height: u32,
        title: &str,
        event_loop: &winit::event_loop::EventLoop<T>,
    ) -> Self {
        let size = winit::dpi::LogicalSize::new(width as f64, height as f64);
        let (width, height): (u32, u32) = size.into();

        let instance: Arc<Instance> = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(
                None,
                Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                &extensions,
                None,
            )
            .expect("failed to create Vulkan instance")
        };

        let cloned_instance = instance.clone();

        let physical: PhysicalDevice = PhysicalDevice::enumerate(&cloned_instance)
            .next()
            .expect("no device available");

        let surface = winit::window::WindowBuilder::new()
            .with_inner_size(size)
            .with_title(title)
            .build_vk_surface(event_loop, instance.clone())
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
            let _alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps
                .supported_formats
                .iter()
                .filter(|&&(fmt, cs)| {
                    fmt.type_color() == Some(NumericType::SRGB) && cs == ColorSpace::SrgbNonLinear
                })
                .map(|&(fmt, _)| fmt)
                .next()
                .expect("failed to find sRGB format");

            (
                Swapchain::start(device.clone(), surface.clone())
                    .num_images(caps.min_image_count)
                    .format(format)
                    .dimensions(surface_dimensions)
                    .usage(ImageUsage::color_attachment())
                    .sharing_mode(&queue)
                    .composite_alpha(caps.supported_composite_alpha.iter().next().unwrap())
                    .build()
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
        let inner_size = self.surface.window().inner_size();
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
            match self.swapchain.recreate().dimensions(new_dimensions).build() {
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
        Some(winit::window::Window::inner_size(self.surface.window()).into())
    }
    fn hidpi_factor(&self) -> f32 {
        winit::window::Window::scale_factor(self.surface.window()) as _
    }
}

// Generate the winit <-> conrod type conversion fns.
conrod_winit::v023_conversion_fns!();
