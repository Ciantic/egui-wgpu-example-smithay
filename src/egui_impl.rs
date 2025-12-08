use smithay_client_toolkit::shell::xdg::window::{Window, WindowConfigure};
use wayland_client::protocol::wl_surface::WlSurface;

use crate::{BaseTrait, CompositorHandlerContainer, KeyboardHandlerContainer, PointerHandlerContainer, WindowContainer};

struct EguiWgpuSurface {
    // TODO: Other stuff?
    pub wl_surface: WlSurface,
}

impl EguiWgpuSurface {
    pub fn new(wl_surface: WlSurface) -> Self {
        // TODO: Other stuff?
        Self { wl_surface }
    }

    pub fn configure(&mut self, width: u32, height: u32) {
    }
}

struct EguiWindow {
    pub surface: EguiWgpuSurface,
}

struct EguiLayerSurface {
    pub surface: EguiWgpuSurface,
}

struct EguiPopup {
    pub surface: EguiWgpuSurface,
}

struct EguiSubsurface {
    pub surface: EguiWgpuSurface,
}


impl CompositorHandlerContainer for EguiWindow {}
impl KeyboardHandlerContainer for EguiWindow {}
impl PointerHandlerContainer for EguiWindow {}
impl BaseTrait for EguiWindow {}
impl WindowContainer for EguiWindow {
    fn configure(
        &mut self,
        configure: &WindowConfigure,
    ) {
        todo!()
    }

    fn get_window(&self) -> &Window {
        todo!()
    }
}