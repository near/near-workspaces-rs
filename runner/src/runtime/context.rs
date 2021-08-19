use std::cell::RefCell;

thread_local! {
    pub static CURRENT_SANDBOX_PORT: RefCell<u16> = RefCell::new(3030);
}

pub(crate) fn current() -> u16 {
    CURRENT_SANDBOX_PORT.with(|ctx| *ctx.borrow())
}

pub(crate) fn enter(port: u16) -> EnterGuard {
    CURRENT_SANDBOX_PORT.with(|ctx| {
        let old = *ctx.borrow();
        *ctx.borrow_mut() = port;
        EnterGuard(old)
    })
}

// EnterGuard used for when entering into a new runtime context then
// after dropping (when runtime ends), goes back to the previous
// runtime context. Used for multi-threading too when a new thread is
// spun up, but currently near Runtimes are single threaded only.
#[derive(Debug)]
pub(crate) struct EnterGuard(u16);

impl Drop for EnterGuard {
    fn drop(&mut self) {
        CURRENT_SANDBOX_PORT.with(|ctx| {
            *ctx.borrow_mut() = self.0;
        });
    }
}
