//! Wakes whoever is consuming background events.
//!
//! While the window is open that is egui (a repaint). While the app runs in the
//! background there is no window, so the wake goes to the event loop instead.

use std::sync::{Arc, Mutex};

type Fallback = Box<dyn Fn() + Send + Sync>;

struct Inner {
    window: Mutex<Option<egui::Context>>,
    fallback: Fallback,
}

#[derive(Clone)]
pub struct Waker(Arc<Inner>);

impl Waker {
    /// `fallback` runs when no window is attached.
    pub fn new(fallback: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(Inner {
            window: Mutex::new(None),
            fallback: Box::new(fallback),
        }))
    }

    /// A waker that only ever repaints an attached window.
    pub fn window_only() -> Self {
        Self::new(|| {})
    }

    pub fn attach(&self, ctx: egui::Context) {
        *self.window() = Some(ctx);
    }

    pub fn detach(&self) {
        *self.window() = None;
    }

    pub fn wake(&self) {
        match self.window().as_ref() {
            Some(ctx) => ctx.request_repaint(),
            None => (self.0.fallback)(),
        }
    }

    fn window(&self) -> std::sync::MutexGuard<'_, Option<egui::Context>> {
        self.0.window.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn falls_back_only_without_a_window() {
        let count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&count);
        let waker = Waker::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });

        waker.wake();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        waker.attach(egui::Context::default());
        waker.wake();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        waker.detach();
        waker.wake();
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
}
