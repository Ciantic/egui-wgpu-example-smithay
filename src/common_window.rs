
use std::num::NonZero;

use log::trace;
use smithay_client_toolkit::{seat::{keyboard::{KeyEvent, Modifiers}, pointer::PointerEvent}, shell::{WaylandSurface, wlr_layer::{LayerSurface, LayerSurfaceConfigure}, xdg::{popup::{Popup, PopupConfigure}, window::{Window, WindowConfigure}}}, shm::{Shm, slot::{Buffer, SlotPool}}};
use wayland_client::{QueueHandle, protocol::{wl_shm, wl_surface::WlSurface}};
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;

use crate::Application;



trait CustomKeyboardHandler {
    fn enter(&mut self) {

    }

    fn leave(&mut self) {

    }

    fn press_key(&mut self, event: KeyEvent) {

    }

    fn release_key(&mut self, event: KeyEvent) {

    }

    fn update_modifiers(&mut self, modifiers: Modifiers) {

    }

    fn repeat_key(&mut self, event: KeyEvent) {
    }
}

trait CustomPointerHandler {
    fn pointer_frame(&mut self, events: &[PointerEvent]) {

    }
}



pub trait WindowContainer {
    fn configure(
        &mut self,
        app: &mut Application,
        configure: WindowConfigure,
    );

    fn request_close(&mut self, app: &mut Application) -> bool;

    fn get_window(&self) -> &Window;
}

pub trait LayerSurfaceContainer {
    fn configure(
        &mut self,
        app: &mut Application,
        config: LayerSurfaceConfigure,
    );

    fn request_close(&mut self, app: &mut Application);

    fn get_layer_surface(&self) -> &LayerSurface;
}

pub trait PopupContainer {
    fn configure(
        &mut self,
        app: &mut Application,
        config: PopupConfigure,
    );

    fn done(&mut self, app: &mut Application);

    fn get_popup(&self) -> &Popup;
}

pub trait SubsurfaceContainer {
    fn configure(&mut self, app: &mut Application, width: u32, height: u32);

    fn get_wl_surface(&self) -> &WlSurface;
}
