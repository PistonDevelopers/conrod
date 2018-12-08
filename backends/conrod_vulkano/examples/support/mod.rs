use std::sync::Arc;
use vulkano::{
    self,
    device::{Device, Queue},
    framebuffer::RenderPassAbstract,
    image::SwapchainImage,
    instance::{Instance, PhysicalDevice},
    swapchain::{PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError},
};

use vulkano_win::{self, VkSurfaceBuild};
use winit::{self, EventsLoop};

pub struct Window {
    pub surface: Arc<Surface<winit::Window>>,
    pub swapchain: Arc<Swapchain<winit::Window>>,
    pub queue: Arc<Queue>,
    pub events_loop: EventsLoop,
    pub device: Arc<Device>,
    pub images: Vec<Arc<SwapchainImage<winit::Window>>>,
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
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

        let events_loop = winit::EventsLoop::new();
        let surface = winit::WindowBuilder::new()
            .with_dimensions(size)
            .with_resizable(false)
            .with_title(title)
            .build_vk_surface(&events_loop, instance.clone())
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
            let format = caps.supported_formats[0].0;

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

        let render_pass = Arc::new(
            single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: vulkano::format::Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );

        Self {
            surface,
            swapchain,
            queue,
            events_loop,
            device,
            images,
            render_pass,
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
