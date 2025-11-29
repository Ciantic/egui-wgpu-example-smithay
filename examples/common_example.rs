use log::trace;

use egui_smithay::*;

use smithay_client_toolkit::{
	compositor::CompositorState, output::OutputState, registry::{ProvidesRegistryState, RegistryState}, seat::SeatState, shell::{wlr_layer::LayerShell, xdg::XdgShell}, shm::Shm
};
use smithay_clipboard::Clipboard;
use wayland_client::{Connection, Proxy, globals::registry_queue_init};

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

	// Create a single xdg window tracked by the Application
	app.create_xdg_window(&compositor_state, &xdg_shell, &qh, "common example");

    app.create_layer_surface(&compositor_state, &layer_shell, &qh, "layer example", None);

	trace!("Starting event loop for common example");

	// Run the Wayland event loop. This example will run until the process is killed
	loop {
		event_queue.blocking_dispatch(&mut app).expect("Wayland dispatch failed");
	}
}
