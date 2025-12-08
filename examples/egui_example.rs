use std::{cell::{RefCell, RefMut}, num::NonZero, rc::Rc};

use egui::{CentralPanel, Context};
use egui_smithay::{Application, EguiWgpuSurface, EguiWindow, ExampleSingleColorWindow, InputState, WindowContainer, egui_app, get_init_app};
use smithay_client_toolkit::{compositor::CompositorState, output::OutputState, registry::RegistryState, seat::{SeatState, pointer::cursor_shape::CursorShapeManager}, shell::{WaylandSurface, wlr_layer::LayerShell, xdg::{XdgShell, window::{Window, WindowConfigure, WindowDecorations}}}, shm::Shm, subcompositor::SubcompositorState};
use smithay_clipboard::Clipboard;
use wayland_client::{Connection, Proxy, QueueHandle, globals::registry_queue_init};

pub struct EguiApp {
    counter: i32,
    text: String,
}

impl EguiApp {
    pub fn new() -> Self {
        Self {
            counter: 0,
            text: String::from("Hello from EGUI!"),
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui WGPU / Smithay example");
            
            ui.separator();
            
            ui.label(format!("Counter: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
            if ui.button("Decrement").clicked() {
                self.counter -= 1;
            }
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Text input:");
                ui.text_edit_singleline(&mut self.text);
            });
            
            ui.label(format!("You wrote: {}", self.text));
            
            ui.separator();
            
            ui.label("This is a simple EGUI app running on Wayland via Smithay toolkit!");
        });
    }
}

impl Default for EguiApp {
    fn default() -> Self {
        Self::new()
    }
}


fn main() {
    env_logger::init();
    let app = get_init_app();

        // Example window --------------------------
    let example_win_surface = app.compositor_state.create_surface(&app.qh);
    let example_window = app.xdg_shell.create_window(
        example_win_surface.clone(),
        WindowDecorations::ServerDefault,
        &app.qh,
    );
    example_window.set_title("Example Window");
    example_window.set_app_id("io.github.smithay.client-toolkit.EguiExample");
    example_window.set_min_size(Some((256, 256)));
    example_window.commit();

    let egui_app = EguiApp::new();

    app.push_window(EguiWindow {
        surface: EguiWgpuSurface::new(example_win_surface),
        // TODO: How to attach egui_app
    });


    app.run_blocking();
}
