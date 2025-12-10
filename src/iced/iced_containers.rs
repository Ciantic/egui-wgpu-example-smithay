use crate::Application;
use crate::BaseTrait;
use crate::CompositorHandlerContainer;
use crate::KeyboardHandlerContainer;
use crate::LayerSurfaceContainer;
use crate::PointerHandlerContainer;
use crate::PopupContainer;
use crate::SubsurfaceContainer;
use crate::WaylandToIcedInput;
use crate::WindowContainer;
use crate::get_app;
use iced::Color;
use iced::Font;
use iced::Pixels;
use iced_graphics::Viewport;
use iced_wgpu::Engine;
use iced_wgpu::Renderer;
use log::trace;
use pollster::block_on;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use smithay_client_toolkit::seat::keyboard::KeyEvent;
use smithay_client_toolkit::seat::keyboard::Modifiers;
use smithay_client_toolkit::seat::pointer::PointerEvent;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure;
use smithay_client_toolkit::shell::xdg::popup::Popup;
use smithay_client_toolkit::shell::xdg::popup::PopupConfigure;
use smithay_client_toolkit::shell::xdg::window::Window;
use smithay_client_toolkit::shell::xdg::window::WindowConfigure;
use smithay_clipboard::Clipboard;
use std::ptr::NonNull;
use wayland_client::Proxy;
use wayland_client::QueueHandle;
use wayland_client::protocol::wl_surface::WlSurface;

struct IcedSurfaceState {
    wl_surface: WlSurface,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    #[allow(dead_code)]
    engine: Engine,
    renderer: Renderer,
    input_state: WaylandToIcedInput,
    queue_handle: QueueHandle<Application>,
    width: u32,
    height: u32,
    scale_factor: i32,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    output_format: wgpu::TextureFormat,
}

impl IcedSurfaceState {
    fn new(wl_surface: WlSurface) -> Self {
        let app = get_app();
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(app.conn.backend().display_ptr() as *mut _)
                .expect("Wayland display pointer was null"),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(wl_surface.id().as_ptr() as *mut _)
                .expect("Wayland surface handle was null"),
        ));

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .expect("Failed to create WGPU surface")
        };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to find a suitable adapter");

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .expect("Failed to request WGPU device");

        let caps = surface.get_capabilities(&adapter);
        let output_format = *caps
            .formats
            .get(0)
            .unwrap_or(&wgpu::TextureFormat::Bgra8Unorm);

        let engine = Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            output_format,
            Some(iced_graphics::Antialiasing::MSAAx4),
            iced_graphics::Shell::headless(),
        );

        let renderer = Renderer::new(engine.clone(), Font::DEFAULT, Pixels(16.0));

        let clipboard = unsafe { Clipboard::new(app.conn.display().id().as_ptr() as *mut _) };
        let input_state = WaylandToIcedInput::new(clipboard);

        Self {
            wl_surface,
            surface,
            device,
            queue,
            engine,
            renderer,
            input_state,
            queue_handle: app.qh.clone(),
            width: 256,
            height: 256,
            scale_factor: 1,
            surface_config: None,
            output_format,
        }
    }

    fn configure(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.input_state.set_screen_size(self.width, self.height);
        self.reconfigure_surface();
        self.render();
    }

    fn frame(&mut self, _time: u32) {
        self.render();
    }

    fn handle_pointer_event(&mut self, _event: &PointerEvent) {
        // WaylandToIcedInput doesn't have handle_pointer_event, need to add it
        // For now, just trigger re-render
        self.render();
    }

    fn handle_keyboard_enter(&mut self) {
        let _ = self.input_state.handle_keyboard_enter();
        self.render();
    }

    fn handle_keyboard_leave(&mut self) {
        let _ = self.input_state.handle_keyboard_leave();
        self.render();
    }

    fn handle_keyboard_event(&mut self, event: &KeyEvent, pressed: bool, repeat: bool) {
        let _ = self
            .input_state
            .handle_keyboard_event(event, pressed, repeat);
        self.render();
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.input_state.update_modifiers(modifiers);
        self.render();
    }

    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.wl_surface.set_buffer_scale(new_factor);
        let factor = new_factor.max(1);
        if factor == self.scale_factor {
            return;
        }
        self.scale_factor = factor;
        self.reconfigure_surface();
        self.render();
    }

    fn render(&mut self) {
        trace!("Rendering surface {}", self.wl_surface.id());
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture");

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let viewport = Viewport::with_physical_size(
            iced::Size::new(self.width, self.height),
            self.physical_scale() as f32,
        );

        // Render the viewport
        self.renderer.present(
            Some(Color::BLACK),
            self.output_format,
            &texture_view,
            &viewport,
        );

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        // Request next frame
        self.wl_surface
            .frame(&self.queue_handle, self.wl_surface.clone());
        self.wl_surface.commit();
    }

    fn reconfigure_surface(&mut self) {
        let width = self.width.saturating_mul(self.physical_scale()).max(1);
        let height = self.height.saturating_mul(self.physical_scale()).max(1);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.output_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![self.output_format],
            desired_maximum_frame_latency: 2,
        };
        self.surface.configure(&self.device, &config);
        self.surface_config = Some(config);
    }

    fn physical_scale(&self) -> u32 {
        self.scale_factor.max(1) as u32
    }
}

pub struct IcedWindow {
    pub window: Window,
    surface: IcedSurfaceState,
}

impl IcedWindow {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        let mut surface = IcedSurfaceState::new(window.wl_surface().clone());
        surface.width = width;
        surface.height = height;
        Self { window, surface }
    }
}

impl CompositorHandlerContainer for IcedWindow {
    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.surface.scale_factor_changed(new_factor);
    }

    fn frame(&mut self, time: u32) {
        self.surface.frame(time);
    }
}

impl KeyboardHandlerContainer for IcedWindow {
    fn enter(&mut self) {
        self.surface.handle_keyboard_enter();
    }

    fn leave(&mut self) {
        self.surface.handle_keyboard_leave();
    }

    fn press_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, false);
    }

    fn release_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, false, false);
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.surface.update_modifiers(modifiers);
    }

    fn repeat_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, true);
    }
}

impl PointerHandlerContainer for IcedWindow {
    fn pointer_frame(&mut self, event: &PointerEvent) {
        self.surface.handle_pointer_event(event);
    }
}

impl BaseTrait for IcedWindow {
    fn get_object_id(&self) -> wayland_backend::client::ObjectId {
        self.window.wl_surface().id()
    }
}

impl WindowContainer for IcedWindow {
    fn configure(&mut self, configure: &WindowConfigure) {
        let width = configure.new_size.0.map_or(256, |size| size.get());
        let height = configure.new_size.1.map_or(256, |size| size.get());
        self.window
            .wl_surface()
            .set_buffer_scale(self.surface.scale_factor);
        self.surface.configure(width, height);
    }
}

pub struct IcedLayerSurface {
    pub layer_surface: LayerSurface,
    surface: IcedSurfaceState,
}

impl IcedLayerSurface {
    pub fn new(layer_surface: LayerSurface, width: u32, height: u32) -> Self {
        let mut surface = IcedSurfaceState::new(layer_surface.wl_surface().clone());
        surface.width = width;
        surface.height = height;
        Self {
            layer_surface,
            surface,
        }
    }
}

impl CompositorHandlerContainer for IcedLayerSurface {
    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.surface.scale_factor_changed(new_factor);
    }

    fn frame(&mut self, time: u32) {
        self.surface.frame(time);
    }
}

impl KeyboardHandlerContainer for IcedLayerSurface {
    fn enter(&mut self) {
        self.surface.handle_keyboard_enter();
    }

    fn leave(&mut self) {
        self.surface.handle_keyboard_leave();
    }

    fn press_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, false);
    }

    fn release_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, false, false);
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.surface.update_modifiers(modifiers);
    }

    fn repeat_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, true);
    }
}

impl PointerHandlerContainer for IcedLayerSurface {
    fn pointer_frame(&mut self, event: &PointerEvent) {
        self.surface.handle_pointer_event(event);
    }
}

impl BaseTrait for IcedLayerSurface {
    fn get_object_id(&self) -> wayland_backend::client::ObjectId {
        self.layer_surface.wl_surface().id()
    }
}

impl LayerSurfaceContainer for IcedLayerSurface {
    fn configure(&mut self, config: &LayerSurfaceConfigure) {
        self.layer_surface
            .wl_surface()
            .set_buffer_scale(self.surface.scale_factor);
        self.surface.configure(config.new_size.0, config.new_size.1);
    }
}

pub struct IcedPopup {
    pub popup: Popup,
    surface: IcedSurfaceState,
}

impl IcedPopup {
    pub fn new(popup: Popup, width: u32, height: u32) -> Self {
        let mut surface = IcedSurfaceState::new(popup.wl_surface().clone());
        surface.width = width;
        surface.height = height;
        Self { popup, surface }
    }
}

impl CompositorHandlerContainer for IcedPopup {
    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.surface.scale_factor_changed(new_factor);
    }

    fn frame(&mut self, time: u32) {
        self.surface.frame(time);
    }
}

impl KeyboardHandlerContainer for IcedPopup {
    fn enter(&mut self) {
        self.surface.handle_keyboard_enter();
    }

    fn leave(&mut self) {
        self.surface.handle_keyboard_leave();
    }

    fn press_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, false);
    }

    fn release_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, false, false);
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.surface.update_modifiers(modifiers);
    }

    fn repeat_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, true);
    }
}

impl PointerHandlerContainer for IcedPopup {
    fn pointer_frame(&mut self, event: &PointerEvent) {
        self.surface.handle_pointer_event(event);
    }
}

impl BaseTrait for IcedPopup {
    fn get_object_id(&self) -> wayland_backend::client::ObjectId {
        self.popup.wl_surface().id()
    }
}

impl PopupContainer for IcedPopup {
    fn configure(&mut self, config: &PopupConfigure) {
        self.popup
            .wl_surface()
            .set_buffer_scale(self.surface.scale_factor);
        self.surface
            .configure(config.width as u32, config.height as u32);
    }

    fn done(&mut self) {}
}

pub struct IcedSubsurface {
    pub wl_surface: WlSurface,
    surface: IcedSurfaceState,
}

impl IcedSubsurface {
    pub fn new(wl_surface: WlSurface, width: u32, height: u32) -> Self {
        let mut surface = IcedSurfaceState::new(wl_surface.clone());
        surface.width = width;
        surface.height = height;
        Self {
            wl_surface,
            surface,
        }
    }
}

impl CompositorHandlerContainer for IcedSubsurface {
    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.surface.scale_factor_changed(new_factor);
    }

    fn frame(&mut self, time: u32) {
        self.surface.frame(time);
    }
}

impl KeyboardHandlerContainer for IcedSubsurface {
    fn enter(&mut self) {
        self.surface.handle_keyboard_enter();
    }

    fn leave(&mut self) {
        self.surface.handle_keyboard_leave();
    }

    fn press_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, false);
    }

    fn release_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, false, false);
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.surface.update_modifiers(modifiers);
    }

    fn repeat_key(&mut self, event: &KeyEvent) {
        self.surface.handle_keyboard_event(event, true, true);
    }
}

impl PointerHandlerContainer for IcedSubsurface {
    fn pointer_frame(&mut self, event: &PointerEvent) {
        self.surface.handle_pointer_event(event);
    }
}

impl BaseTrait for IcedSubsurface {
    fn get_object_id(&self) -> wayland_backend::client::ObjectId {
        self.wl_surface.id()
    }
}

impl SubsurfaceContainer for IcedSubsurface {
    fn configure(&mut self, width: u32, height: u32) {
        self.wl_surface.set_buffer_scale(self.surface.scale_factor);
        self.surface.configure(width, height);
    }
}
