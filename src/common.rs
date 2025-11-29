use log::trace;
use smithay_client_toolkit::{compositor::CompositorHandler, delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer, delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window, output::{OutputHandler, OutputState}, registry::{ProvidesRegistryState, RegistryState}, registry_handlers, seat::{Capability, SeatHandler, SeatState, keyboard::{KeyEvent, KeyboardHandler}, pointer::{PointerEvent, PointerHandler, ThemedPointer}}, shell::{WaylandSurface, wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure}, xdg::window::{Window, WindowHandler}}, shm::{Shm, ShmHandler, slot::{Buffer, SlotPool}}};
use wayland_client::{Connection, QueueHandle, Proxy, protocol::{wl_output, wl_seat, wl_shm, wl_surface::WlSurface}};

use crate::InputState;

pub enum WindowKind {
    LayerSurface(LayerSurface),
    Window(Window),
}

pub struct WaylandWindow {
    pub width: u32,
    pub height: u32,
    pub scale_factor: i32,
    pub themed_pointer: Option<ThemedPointer>,
    pub kind: WindowKind,
    pub buffer: Option<Buffer>,
}

pub struct Application {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm_state: Shm,
    pub windows: Vec<WaylandWindow>,
    pub input_state: InputState,
    // Pool used to create shm buffers for simple software presentation in examples
    pub pool: Option<SlotPool>,
}

impl Application {
    /// Create a new Application container from the provided globals state pieces.
    pub fn new(registry_state: RegistryState, seat_state: SeatState, output_state: OutputState, shm_state: Shm, input_state: InputState) -> Self {
        Self {
            registry_state,
            seat_state,
            output_state,
            shm_state,
            windows: Vec::new(),
            input_state,
            pool: None,
        }
    }

    /// Create a simple layer surface and track it in `self.windows`.
    pub fn create_layer_surface(&mut self, compositor_state: &smithay_client_toolkit::compositor::CompositorState, layer_shell: &smithay_client_toolkit::shell::wlr_layer::LayerShell, qh: &QueueHandle<Self>, name: &str, chosen_output: Option<&wl_output::WlOutput>) {
        let surface = compositor_state.create_surface(qh);
        let layer_surface = layer_shell.create_layer_surface(
            qh,
            surface,
            smithay_client_toolkit::shell::wlr_layer::Layer::Top,
            Some(name),
            chosen_output,
        );
        // Make the layer visible and interactive by anchoring and enabling keyboard interactivity
        layer_surface.set_anchor(smithay_client_toolkit::shell::wlr_layer::Anchor::BOTTOM);
        layer_surface.set_keyboard_interactivity(smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::OnDemand);
        layer_surface.set_size(256, 256);
        layer_surface.commit();

        trace!("[COMMON] Created layer surface '{}'", name);

        let win = WaylandWindow {
            width: 256,
            height: 256,
            scale_factor: 1,
            themed_pointer: None,
            kind: WindowKind::LayerSurface(layer_surface),
            buffer: None,
        };
        self.windows.push(win);
    }

    /// Create a simple xdg window and track it in `self.windows`.
    pub fn create_xdg_window(&mut self, compositor_state: &smithay_client_toolkit::compositor::CompositorState, xdg_shell: &smithay_client_toolkit::shell::xdg::XdgShell, qh: &QueueHandle<Self>, title: &str) {
        let surface = compositor_state.create_surface(qh);
        let window = xdg_shell.create_window(surface, smithay_client_toolkit::shell::xdg::window::WindowDecorations::ServerDefault, qh);
        window.set_title(title);
        window.set_app_id("io.github.smithay.client-toolkit.EguiExample");
        window.set_min_size(Some((256, 256)));
        window.commit();

        let win = WaylandWindow {
            width: 256,
            height: 256,
            scale_factor: 1,
            themed_pointer: None,
            kind: WindowKind::Window(window),
            buffer: None,
        };
        self.windows.push(win);
    }

    /// List textual descriptions of tracked windows.
    pub fn list_windows(&self) -> Vec<String> {
        self.windows.iter().enumerate().map(|(i,w)| {
            match &w.kind {
                WindowKind::LayerSurface(_) => format!("{}: LayerSurface ({}x{})", i, w.width, w.height),
                WindowKind::Window(_) => format!("{}: XDG Window ({}x{})", i, w.width, w.height),
            }
        }).collect()
    }

    /// Close and remove the window at index, if any.
    pub fn close_window(&mut self, index: usize) -> Option<()> {
        if index < self.windows.len() {
            // Dropping the wrapper will drop the underlying Wayland object
            self.windows.remove(index);
            Some(())
        } else {
            None
        }
    }
}

impl CompositorHandler for Application {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        new_factor: i32,
    ) {
        trace!("[MAIN] Scale factor changed to {}", new_factor);

        _surface.frame(qh, _surface.clone());
        _surface.commit();
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _time: u32,
    ) {
        trace!("[MAIN] Frame callback");
        // self.render(conn, qh);
        // if needs_repaint {

        // This would render in loop:
        // _surface.damage_buffer(0, 0, 256, 256);
        // _surface.frame(qh, _surface.clone());
        // _surface.commit();
        // }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl OutputHandler for Application {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Application {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        // self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        target_layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // When the compositor sends a configure we should attach a buffer
        // so the compositor can map the window. Create a SlotPool on first
        // configure and draw a simple solid buffer to make the window visible.
        trace!("[COMMON] XDG layer configure");

        // Extract size from configure
        let new_width = configure.new_size.0;
        let new_height = configure.new_size.1;

        // Find the matching tracked window and attach a simple shm buffer
        for win in &mut self.windows {
            if let WindowKind::LayerSurface(layer) = &mut win.kind {
                if layer.wl_surface().id().as_ptr() == target_layer.wl_surface().id().as_ptr() {
                    win.width = new_width;
                    win.height = new_height;
                    self.input_state.set_screen_size(win.width, win.height);

                    // Ensure pool exists
                    if self.pool.is_none() {
                        let size = (win.width as usize) * (win.height as usize) * 4;
                        self.pool = Some(SlotPool::new(size, &self.shm_state).expect("Failed to create SlotPool"));
                    }

                    let pool = self.pool.as_mut().unwrap();
                    let stride = win.width as i32 * 4;

                    // Create a buffer and paint it a simple color
                    let (buffer, _maybe_canvas) = pool.create_buffer(win.width as i32, win.height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
                    if let Some(canvas) = pool.canvas(&buffer) {
                        for chunk in canvas.chunks_exact_mut(4) {
                            // ARGB little-endian: B, G, R, A
                            chunk[0] = 0x30; // B
                            chunk[1] = 0x80; // G
                            chunk[2] = 0xC0; // R
                            chunk[3] = 0xFF; // A
                        }
                    }

                    // Damage, frame and attach
                    target_layer.wl_surface().damage_buffer(0, 0, win.width as i32, win.height as i32);
                    target_layer.wl_surface().frame(qh, target_layer.wl_surface().clone());
                    buffer.attach_to(target_layer.wl_surface()).expect("buffer attach");
                    target_layer.wl_surface().commit();

                    win.buffer = Some(buffer);
                    break;
                }
            }
        }
    }
}

impl WindowHandler for Application {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        // No-op for this simple helper container
        trace!("[COMMON] XDG window close requested");
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        target_window: &Window,
        configure: smithay_client_toolkit::shell::xdg::window::WindowConfigure,
        _serial: u32,
    ) {
        // When the compositor sends a configure we should attach a buffer
        // so the compositor can map the window. Create a SlotPool on first
        // configure and draw a simple solid buffer to make the window visible.
        trace!("[COMMON] XDG window configure");

        // Extract size from configure
        let new_width = configure.new_size.0.map_or(256, |v| v.get());
        let new_height = configure.new_size.1.map_or(256, |v| v.get());

        // Find the matching tracked window and attach a simple shm buffer
        for win in &mut self.windows {
            if let WindowKind::Window(wnd) = &mut win.kind {
                if wnd.wl_surface().id().as_ptr() == target_window.wl_surface().id().as_ptr() {
                    win.width = new_width;
                    win.height = new_height;
                    self.input_state.set_screen_size(win.width, win.height);

                    // Ensure pool exists
                    if self.pool.is_none() {
                        let size = (win.width as usize) * (win.height as usize) * 4;
                        self.pool = Some(SlotPool::new(size, &self.shm_state).expect("Failed to create SlotPool"));
                    }

                    let pool = self.pool.as_mut().unwrap();
                    let stride = win.width as i32 * 4;

                    // Create a buffer and paint it a simple color
                    let (buffer, _maybe_canvas) = pool.create_buffer(win.width as i32, win.height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");
                    if let Some(canvas) = pool.canvas(&buffer) {
                        for chunk in canvas.chunks_exact_mut(4) {
                            // ARGB little-endian: B, G, R, A
                            chunk[0] = 0x30; // B
                            chunk[1] = 0x80; // G
                            chunk[2] = 0xC0; // R
                            chunk[3] = 0xFF; // A
                        }
                    }

                    // Damage, frame and attach
                    wnd.wl_surface().damage_buffer(0, 0, win.width as i32, win.height as i32);
                    wnd.wl_surface().frame(qh, wnd.wl_surface().clone());
                    buffer.attach_to(wnd.wl_surface()).expect("buffer attach");
                    wnd.commit();

                    win.buffer = Some(buffer);
                    break;
                }
            }
        }
    }
}

impl PointerHandler for Application {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        trace!("[MAIN] Pointer frame with {} events", events.len());
        for event in events {
            self.input_state.handle_pointer_event(event);
        }
        // Request a redraw after input
        trace!("[MAIN] Requesting frame after pointer input");
        // self.layer_surface.wl_surface().frame(&_qh, self.layer_surface.wl_surface().clone());
        // self.layer_surface.wl_surface().commit();
    }
}

impl KeyboardHandler for Application {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym],
    ) {
        trace!("[MAIN] Keyboard focus gained");
        // Keyboard focus gained
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
    ) {
        trace!("[MAIN] Keyboard focus lost");
        // Keyboard focus lost
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        trace!("[MAIN] Key pressed");

        
        self.input_state.handle_keyboard_event(&event, true, false);
        
        // Request a redraw after input
        trace!("[MAIN] Requesting frame after key press");
        // self.layer_surface.wl_surface().frame(&_qh, self.layer_surface.wl_surface().clone());
        // self.layer_surface.wl_surface().commit();
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        self.input_state.handle_keyboard_event(&event, false, false);
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: smithay_client_toolkit::seat::keyboard::Modifiers,
        _raw_modifiers: smithay_client_toolkit::seat::keyboard::RawModifiers,
        _layout: u32,
    ) {
        self.input_state.update_modifiers(&modifiers);
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        self.input_state.handle_keyboard_event(&event, true, true);
        // Request a redraw after input
        // self.layer_surface.wl_surface().frame(&_qh, self.layer_surface.wl_surface().clone());
        // self.layer_surface.wl_surface().commit();
    }
}

impl SeatHandler for Application {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        trace!("[MAIN] New seat capability: {:?}", capability);
        if capability == Capability::Keyboard && self.seat_state.get_keyboard(qh, &seat, None).is_err() {
            trace!("[MAIN] Failed to get keyboard");
        }
        // if capability == Capability::Pointer && self.themed_pointer.is_none() {
        //     trace!("[MAIN] Creating themed pointer");
        //     let surface = self.layer_surface.wl_surface().clone();
        //     match self.seat_state.get_pointer_with_theme(
        //         qh,
        //         &seat,
        //         self.shm_state.wl_shm(),
        //         surface,
        //         ThemeSpec::default(),
        //     ) {
        //         Ok(themed_pointer) => {
        //             self.themed_pointer = Some(themed_pointer);
        //             trace!("[MAIN] Themed pointer created successfully");
        //         }
        //         Err(e) => {
        //             trace!("[MAIN] Failed to create themed pointer: {:?}", e);
        //         }
        //     }
        // }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl ShmHandler for Application {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl ProvidesRegistryState for Application {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(Application);
delegate_output!(Application);
delegate_shm!(Application);

delegate_seat!(Application);
delegate_keyboard!(Application);
delegate_pointer!(Application);

delegate_layer!(Application);

delegate_xdg_shell!(Application);
delegate_xdg_window!(Application);

delegate_registry!(Application);
