use std::sync::Arc;

pub struct Context {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub window: Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'static>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface_format: wgpu::TextureFormat,
    surface_config: wgpu::SurfaceConfiguration,
}

impl Context {
    pub async fn new(window: Arc<winit::window::Window>, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let required_features = if adapter.features().contains(wgpu::Features::BGRA8UNORM_STORAGE) {
            wgpu::Features::BGRA8UNORM_STORAGE
        } else {
            wgpu::Features::empty()
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let size = winit::dpi::PhysicalSize { width, height };
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        
        let has_bgra8unorm_storage = adapter.features().contains(wgpu::Features::BGRA8UNORM_STORAGE);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Rgba8Unorm || (has_bgra8unorm_storage && *f == wgpu::TextureFormat::Bgra8Unorm))
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Self {
            instance,
            device,
            queue,
            window,
            surface,
            size,
            surface_format,
            surface_config,
        }
    }
}
