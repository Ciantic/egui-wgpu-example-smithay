use iced::Element;
use iced::widget::button;
use iced::widget::column;
use iced::widget::text;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::xdg::window::WindowDecorations;
use wayapp::IcedAppData;
use wayapp::IcedWindow;
use wayapp::get_init_app;

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl IcedAppData for Counter {
    type Message = Message;

    fn view(&self) -> Element<Message> {
        column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement),
        ]
        .padding(20)
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }
}

fn main() {
    env_logger::init();
    let app = get_init_app();

    // Create a window surface
    let window_surface = app.compositor_state.create_surface(&app.qh);
    let window =
        app.xdg_shell
            .create_window(window_surface, WindowDecorations::ServerDefault, &app.qh);
    window.set_title("Iced Counter Example");
    window.set_app_id("io.github.ciantic.wayapp.IcedExample");
    window.set_min_size(Some((256, 256)));
    window.commit();

    // Create an Iced counter app and window
    let counter = Counter::default();
    let iced_window = IcedWindow::new(window, counter, 256, 256);
    app.push_window(iced_window);

    // Run the application event loop
    app.run_blocking();
}
