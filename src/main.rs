mod buffers;
mod context;
mod pipeline;

use crate::buffers::Texture;
use crate::{buffers::StorageTexture, context::Context, pipeline::ComputePipeline};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wgpu::BindGroupEntry;
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy},
    window::{Window, WindowId},
};

struct App {
    proxy: EventLoopProxy<AppEvent>,
    context: Option<Context>,
    pipeline: Option<ComputePipeline>,
    output: Option<StorageTexture>,
    input_texture: Option<Texture>,
    bind_group: Option<wgpu::BindGroup>,
    scratch_canvas: Option<web_sys::HtmlCanvasElement>,
    scratch_ctx: Option<web_sys::CanvasRenderingContext2d>,
}

impl App {
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        Self {
            proxy,
            context: None,
            pipeline: None,
            output: None,
            input_texture: None,
            bind_group: None,
            scratch_canvas: None,
            scratch_ctx: None,
        }
    }

    pub fn update(&mut self, _event: &winit::event::KeyEvent) {
        if let Some(context) = self.context.as_mut() {
            context.window.request_redraw();
        }
    }

    pub fn render(&mut self) {
        let output = match self.context.as_ref().unwrap().surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => {
                log::error!("Failed to acquire next swap chain texture");
                return;
            }
        };

        if self.fit_input_to_video() {
            let bind_group = self.context.as_ref().unwrap().device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Video to ascii bind group"),
                    layout: &self.pipeline.as_ref().unwrap().bind_groups_layouts[0],
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.output.as_ref().unwrap().view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &self.input_texture.as_ref().unwrap().view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(
                                &self.input_texture.as_ref().unwrap().sampler,
                            ),
                        },
                    ],
                },
            );

            self.bind_group = Some(bind_group);
        }

        let video_player = if let Some(video) = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("video-player")
            && let Ok(video_player) = video.dyn_into::<web_sys::HtmlVideoElement>()
        {
            video_player
        } else {
            log::error!("Video element not found");
            return;
        };

        if video_player.ready_state() >= 2 {
            let scratch_canvas = self.scratch_canvas.as_ref().unwrap();
            let scratch_ctx = self.scratch_ctx.as_ref().unwrap();

            let target_w = self.input_texture.as_ref().unwrap().extent.width;
            let target_h = self.input_texture.as_ref().unwrap().extent.height;

            if scratch_canvas.width() != target_w {
                scratch_canvas.set_width(target_w);
            }
            if scratch_canvas.height() != target_h {
                scratch_canvas.set_height(target_h);
            }

            let _ = scratch_ctx.draw_image_with_html_video_element(&video_player, 0.0, 0.0);

            let source = wgpu::CopyExternalImageSourceInfo {
                source: wgpu::ExternalImageSource::HTMLCanvasElement(scratch_canvas.clone()),
                origin: wgpu::Origin2d::ZERO,
                flip_y: false,
            };
            let dest = wgpu::CopyExternalImageDestInfo {
                texture: &self.input_texture.as_ref().unwrap().texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
                color_space: wgpu::PredefinedColorSpace::Srgb,
                premultiplied_alpha: false,
            };

            self.context
                .as_ref()
                .unwrap()
                .queue
                .copy_external_image_to_texture(
                    &source,
                    dest,
                    wgpu::Extent3d {
                        width: self.input_texture.as_ref().unwrap().extent.width,
                        height: self.input_texture.as_ref().unwrap().extent.height,
                        depth_or_array_layers: 1,
                    },
                );
        }

        let mut encoder = self
            .context
            .as_ref()
            .unwrap()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Video to ascii compute Pass"),
                ..Default::default()
            });

            compute_pass.set_pipeline(&self.pipeline.as_ref().unwrap().pipeline);
            compute_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

            let workgroup_count_x = self.context.as_ref().unwrap().size.width.div_ceil(16);
            let workgroup_count_y = self.context.as_ref().unwrap().size.height.div_ceil(16);
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        encoder.copy_texture_to_texture(
            self.output.as_ref().unwrap().texture.as_image_copy(),
            output.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.context.as_ref().unwrap().size.width,
                height: self.context.as_ref().unwrap().size.height,
                depth_or_array_layers: 1,
            },
        );

        self.context
            .as_ref()
            .unwrap()
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn fit_input_to_video(&mut self) -> bool {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(video) = document.get_element_by_id("video-player")
            && let Ok(video_player) = video.dyn_into::<web_sys::HtmlVideoElement>()
        {
            let vw = video_player.video_width();
            let vh = video_player.video_height();

            let cw = self
                .input_texture
                .as_ref()
                .map(|t| t.extent.width)
                .unwrap_or(0);
            let ch = self
                .input_texture
                .as_ref()
                .map(|t| t.extent.height)
                .unwrap_or(0);

            if vw > 0 && vh > 0 && (cw != vw || ch != vh) {
                self.input_texture = Some(Texture::new(
                    vw,
                    vh,
                    self.context.as_ref().unwrap(),
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    Some("Input video texture"),
                ));

                log::info!("Resized input texture to match video dimensions: {}x{}", vw, vh);

                return true;
            }
        }
        false
    }
}

enum AppEvent {
    WindowInitialized(Context),
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let proxy = self.proxy.clone();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("viewport-canvas")
            .expect("Canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let width = canvas.client_width();
        let height = canvas.client_height();
        let attributes = Window::default_attributes()
            .with_active(false)
            .with_canvas(Some(canvas.clone()))
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        log::info!("Created output window with size: {}x{}", width, height);

        if let Some(video_input) = document.get_element_by_id("video-input") {
            if let Some(video_player) = document.get_element_by_id("video-player") {
                let video_player: web_sys::HtmlVideoElement = video_player.dyn_into().unwrap();
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let target = event.target().unwrap();
                    let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    if let Some(files) = input.files() {
                        if files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let url = web_sys::Url::create_object_url_with_blob(&file).unwrap();
                            video_player.set_src(&url);
                            let _ = video_player.play();
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                video_input
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }
        }

        wasm_bindgen_futures::spawn_local(async move {
            let context = Context::new(
                window.clone(),
                width.try_into().unwrap(),
                height.try_into().unwrap(),
            )
            .await;
            let _ = proxy.send_event(AppEvent::WindowInitialized(context));
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::WindowInitialized(context) => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document
                    .create_element("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();
                self.scratch_canvas = Some(canvas);
                self.scratch_ctx = Some(ctx);

                let pipeline = ComputePipeline::new(&context, Some("Video to ASCII pipeline"));
                self.pipeline = Some(pipeline);

                self.context = Some(context);

                self.output = Some(StorageTexture::new(
                    self.context.as_ref().unwrap().size.width,
                    self.context.as_ref().unwrap().size.height,
                    self.context.as_ref().unwrap(),
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::STORAGE_BINDING,
                    Some("Output Texture"),
                ));

                let input_texture = Texture::new(
                    1,
                    1,
                    self.context.as_ref().unwrap(),
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    Some("Input video texture"),
                );

                let bind_group = self.context.as_ref().unwrap().device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("Video to ascii bind group"),
                        layout: &self.pipeline.as_ref().unwrap().bind_groups_layouts[0],
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.output.as_ref().unwrap().view,
                                ),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&input_texture.view),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(&input_texture.sampler),
                            },
                        ],
                    },
                );

                self.input_texture = Some(input_texture);
                self.bind_group = Some(bind_group);

                self.context.as_ref().unwrap().window.request_redraw();

                log::info!("Initialization complete, starting render loop");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.update(&event);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(context) = self.context.as_ref() {
            context.window.request_redraw();
        }
    }
}

#[wasm_bindgen(start)]
pub async fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize console logger");

    let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event()
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();
    let app = App::new(proxy);

    event_loop.set_control_flow(ControlFlow::Poll);

    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);
}

fn main() {}
