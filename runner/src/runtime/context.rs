use std::cell::RefCell;

thread_local! {
    pub static CURRENT_SANDBOX_PORT: RefCell<u16> = RefCell::new(3030);
}

pub(crate) fn current() -> u16 {
    CURRENT_SANDBOX_PORT.with(|ctx| *ctx.borrow())
}

pub(crate) fn enter(port: u16) {
    CURRENT_SANDBOX_PORT.with(|ctx| {
        *ctx.borrow_mut() = port;
    });
}
