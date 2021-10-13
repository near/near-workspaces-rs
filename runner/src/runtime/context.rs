use super::{RuntimeFlavor, SandboxRuntime, TestnetRuntime};
use std::cell::RefCell;
use std::str::FromStr;

pub const MISSING_RUNTIME_ERROR: &str =
    "there is no runtime running: need to be ran from a NEAR runtime context";

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

/// Spawn this task within a new runtime context. Useful for when trying to
/// run multiple runtimes (testnet, sandbox, ...) within the same thread.
// NOTE: this could also be equivalent to tokio::spawn as well
pub async fn within<T>(runtime: &str, scoped_task: T) -> anyhow::Result<T::Output>
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    let rt_flavor = RuntimeFlavor::from_str(runtime)?;
    let task = move || {
        // In hindsight, this look bad doing it this way, where we create {Sandbox, Testnet}Runtime
        // to do mutable runtime context switching with `enter` function.
        match rt_flavor {
            RuntimeFlavor::Sandbox(_) => {
                let mut rt = SandboxRuntime::default();
                let _ = rt.run().unwrap();

                tokio::runtime::Handle::current().block_on(scoped_task)
            }
            RuntimeFlavor::Testnet => {
                let mut rt = TestnetRuntime::default();
                let _ = rt.run().unwrap();

                tokio::runtime::Handle::current().block_on(scoped_task)
            }
            _ => unimplemented!(),
        }
    };

    tokio::task::spawn_blocking(task)
        .await
        .map_err(|e| e.into())
}
