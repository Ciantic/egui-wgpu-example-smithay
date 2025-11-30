use log::trace;

use egui_smithay::*;

use smithay_client_toolkit::{
	compositor::CompositorState, output::OutputState, registry::{ProvidesRegistryState, RegistryState}, seat::SeatState, shell::{WaylandSurface, wlr_layer::{Anchor, Layer, LayerShell}, xdg::{XdgShell, window::WindowDecorations}}, shm::Shm
};
use smithay_clipboard::Clipboard;
use wayland_client::{Connection, Proxy, globals::registry_queue_init};
use wayland_protocols::xdg::shell::client::xdg_popup::XdgPopup;

fn main() {
	env_logger::init();

	let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
	let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
	let qh = event_queue.handle();

	// Bind required globals
	let compositor_state = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
	let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell not available");
	let shm_state = Shm::bind(&globals, &qh).expect("wl_shm not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");

	// Clipboard (needed for InputState)
	let clipboard = unsafe { Clipboard::new(conn.display().id().as_ptr() as *mut _) };

	// Build our Application container from `common.rs`
	let mut app = Application::new(
		RegistryState::new(&globals),
		SeatState::new(&globals, &qh),
		OutputState::new(&globals, &qh),
		shm_state,
		InputState::new(clipboard),
	);

	// Experiment to share the same surface between multiple layer surfaces
	let shared_surface = compositor_state.create_surface(&qh);

	let example_layer_surface = layer_shell.create_layer_surface(
		&qh,
		shared_surface.clone(),
		Layer::Top,
		Some("Example"),
		None,
	);
	example_layer_surface.set_anchor(Anchor::BOTTOM | Anchor::LEFT);
	example_layer_surface.set_margin(0, 0, 20, 20);
	example_layer_surface.set_size(256, 256);
	example_layer_surface.commit();

	let example_layer_surface2 = layer_shell.create_layer_surface(
		&qh,
		shared_surface.clone(),
		Layer::Top,
		Some("Example2"),
		None,
	);
	example_layer_surface2.set_anchor(Anchor::BOTTOM | Anchor::RIGHT);
	example_layer_surface2.set_margin(0, 20, 20, 0);
	example_layer_surface2.set_size(512, 256);
	example_layer_surface2.commit();

	// Crazy experiment to share the same surface between multiple windows
	let shared_win_surface = compositor_state.create_surface(&qh);

	let example_window = xdg_shell.create_window(shared_win_surface.clone(), WindowDecorations::ServerDefault, &qh);
	example_window.set_title("Example Window");
	example_window.set_app_id("io.github.smithay.client-toolkit.EguiExample");
	example_window.set_min_size(Some((256,256)));
	example_window.commit();

	let example_window2 = xdg_shell.create_window(shared_win_surface.clone(), WindowDecorations::ServerDefault, &qh);
	example_window2.set_title("Example Window 2");
	example_window2.set_app_id("io.github.smithay.client-toolkit.EguiExample2");
	example_window2.set_min_size(Some((256,256)));
	example_window2.commit();


	trace!("Starting event loop for common example");

	// Run the Wayland event loop. This example will run until the process is killed
	loop {
		event_queue.blocking_dispatch(&mut app).expect("Wayland dispatch failed");
	}
}
