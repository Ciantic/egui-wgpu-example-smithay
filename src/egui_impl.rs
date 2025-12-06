use wayland_client::protocol::wl_surface::WlSurface;

struct EguiSurface {
    pub wl_surface: WlSurface,

}

struct EguiWindow {
    pub surface: EguiSurface,
}

struct EguiLayerSurface {
    pub surface: EguiSurface,
}

struct EguiPopup {
    pub surface: EguiSurface,
}

struct EguiSubsurface {
    pub surface: EguiSurface,
}