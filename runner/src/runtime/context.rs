use std::cell::RefCell;
use super::RuntimeFlavor;


thread_local! {
    static RT_CONTEXT: RefCell<Option<RuntimeFlavor>> = RefCell::new(None);
}

pub(crate) fn current() -> Option<RuntimeFlavor> {
    RT_CONTEXT.with(|ctx| ctx.borrow().clone())
}

pub(crate) fn enter(flavor: RuntimeFlavor) -> EnterGuard {
    RT_CONTEXT.with(|ctx| {
        let old = ctx.borrow_mut().replace(flavor);
        EnterGuard(old)
    })
}

// EnterGuard used for when entering into a new runtime context then
// after dropping (when runtime ends), goes back to the previous
// runtime context. Used for multi-threading too when a new thread is
// spun up, but currently near Runtimes are single threaded only.
#[derive(Debug)]
pub(crate) struct EnterGuard(Option<RuntimeFlavor>);

impl Drop for EnterGuard {
    fn drop(&mut self) {
        RT_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = self.0.take();
        });
    }
}
