use super::Window;

#[allow(unused_variables)]
pub trait Events {
    fn on_new_window(&self, window: Window) {}
}
