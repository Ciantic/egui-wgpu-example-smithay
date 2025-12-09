#![allow(unused_variables)]

use smithay_client_toolkit::seat::keyboard::KeyEvent;
use smithay_client_toolkit::seat::keyboard::Modifiers;
use smithay_client_toolkit::seat::pointer::PointerEvent;
use smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure;
use smithay_client_toolkit::shell::xdg::popup::PopupConfigure;
use smithay_client_toolkit::shell::xdg::window::WindowConfigure;
use std::cell::RefCell;
use std::rc::Rc;
use wayland_backend::client::ObjectId;
use wayland_client::protocol::wl_output::Transform;
use wayland_client::protocol::wl_output::WlOutput;

pub trait KeyboardHandlerContainer {
    fn enter(&mut self) {}

    fn leave(&mut self) {}

    fn press_key(&mut self, event: &KeyEvent) {}

    fn release_key(&mut self, event: &KeyEvent) {}

    fn update_modifiers(&mut self, modifiers: &Modifiers) {}

    fn repeat_key(&mut self, event: &KeyEvent) {}
}

pub trait PointerHandlerContainer {
    fn pointer_frame(&mut self, events: &PointerEvent) {}
}

pub trait CompositorHandlerContainer {
    fn scale_factor_changed(&mut self, new_factor: i32) {}

    fn transform_changed(&mut self, new_transform: &Transform) {}

    fn frame(&mut self, time: u32) {}

    fn surface_enter(&mut self, output: &WlOutput) {}

    fn surface_leave(&mut self, output: &WlOutput) {}
}

pub trait BaseTrait:
    CompositorHandlerContainer + KeyboardHandlerContainer + PointerHandlerContainer
{
}

pub trait WindowContainer: BaseTrait {
    fn configure(&mut self, configure: &WindowConfigure);

    fn get_object_id(&self) -> ObjectId;

    fn allowed_to_close(&self) -> bool {
        true
    }

    fn request_close(&mut self) {}
}

pub trait LayerSurfaceContainer: BaseTrait {
    fn configure(&mut self, config: &LayerSurfaceConfigure);

    fn closed(&mut self) {}

    fn get_object_id(&self) -> ObjectId;
}

pub trait PopupContainer: BaseTrait {
    fn configure(&mut self, config: &PopupConfigure);

    fn done(&mut self) {}

    fn get_object_id(&self) -> ObjectId;
}

pub trait SubsurfaceContainer: BaseTrait {
    fn configure(&mut self, width: u32, height: u32);

    fn get_object_id(&self) -> ObjectId;
}

// Blanket implementations for Rc<RefCell<T>> to allow shared mutable access
impl<T: KeyboardHandlerContainer + ?Sized> KeyboardHandlerContainer for Rc<RefCell<T>> {
    fn enter(&mut self) {
        self.borrow_mut().enter();
    }

    fn leave(&mut self) {
        self.borrow_mut().leave();
    }

    fn press_key(&mut self, event: &KeyEvent) {
        self.borrow_mut().press_key(event);
    }

    fn release_key(&mut self, event: &KeyEvent) {
        self.borrow_mut().release_key(event);
    }

    fn update_modifiers(&mut self, modifiers: &Modifiers) {
        self.borrow_mut().update_modifiers(modifiers);
    }

    fn repeat_key(&mut self, event: &KeyEvent) {
        self.borrow_mut().repeat_key(event);
    }
}

impl<T: PointerHandlerContainer + ?Sized> PointerHandlerContainer for Rc<RefCell<T>> {
    fn pointer_frame(&mut self, events: &PointerEvent) {
        self.borrow_mut().pointer_frame(events);
    }
}

impl<T: CompositorHandlerContainer + ?Sized> CompositorHandlerContainer for Rc<RefCell<T>> {
    fn scale_factor_changed(&mut self, new_factor: i32) {
        self.borrow_mut().scale_factor_changed(new_factor);
    }

    fn transform_changed(&mut self, new_transform: &Transform) {
        self.borrow_mut().transform_changed(new_transform);
    }

    fn frame(&mut self, time: u32) {
        self.borrow_mut().frame(time);
    }

    fn surface_enter(&mut self, output: &WlOutput) {
        self.borrow_mut().surface_enter(output);
    }

    fn surface_leave(&mut self, output: &WlOutput) {
        self.borrow_mut().surface_leave(output);
    }
}

impl<T: BaseTrait + ?Sized> BaseTrait for Rc<RefCell<T>> {}

impl<T: WindowContainer + ?Sized> WindowContainer for Rc<RefCell<T>> {
    fn configure(&mut self, configure: &WindowConfigure) {
        self.borrow_mut().configure(configure);
    }

    fn get_object_id(&self) -> ObjectId {
        self.borrow().get_object_id()
    }

    fn allowed_to_close(&self) -> bool {
        self.borrow().allowed_to_close()
    }

    fn request_close(&mut self) {
        self.borrow_mut().request_close();
    }
}

impl<T: LayerSurfaceContainer + ?Sized> LayerSurfaceContainer for Rc<RefCell<T>> {
    fn configure(&mut self, config: &LayerSurfaceConfigure) {
        self.borrow_mut().configure(config);
    }

    fn closed(&mut self) {
        self.borrow_mut().closed();
    }

    fn get_object_id(&self) -> ObjectId {
        self.borrow().get_object_id()
    }
}

impl<T: PopupContainer + ?Sized> PopupContainer for Rc<RefCell<T>> {
    fn configure(&mut self, config: &PopupConfigure) {
        self.borrow_mut().configure(config);
    }

    fn done(&mut self) {
        self.borrow_mut().done();
    }

    fn get_object_id(&self) -> ObjectId {
        self.borrow().get_object_id()
    }
}

impl<T: SubsurfaceContainer + ?Sized> SubsurfaceContainer for Rc<RefCell<T>> {
    fn configure(&mut self, width: u32, height: u32) {
        self.borrow_mut().configure(width, height);
    }

    fn get_object_id(&self) -> ObjectId {
        self.borrow().get_object_id()
    }
}
