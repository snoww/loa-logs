use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use log::*;
use tauri::{AppHandle, Emitter, Event, EventId, Listener};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait EventManager {
    fn set_boss_only_damage(&self);
    fn has_reset(&self) -> bool;
    fn has_paused(&self) -> bool;
    fn has_saved(&self) -> bool;
    fn can_emit_details(&self) -> bool;
    fn has_toggled_boss_only_damage(&self) -> bool;
}

pub struct TauriEventManager(Arc<EventManagerInner>);

struct EventManagerInner {
    app_handle: AppHandle,
    subscriptions: Mutex<Vec<EventId>>,
    reset: AtomicBool,
    save: AtomicBool,
    pause: AtomicBool,
    boss_only_damage: AtomicBool,
    emit_details: AtomicBool,
}

impl EventManager for TauriEventManager {
    fn set_boss_only_damage(&self) {
        self.0.boss_only_damage.store(true, Ordering::Relaxed);
    }

    fn has_reset(&self) -> bool {
        self.0.has_reset()
    }

    fn has_paused(&self) -> bool {
        self.0.has_paused()
    }

    fn has_saved(&self) -> bool {
        self.0.has_saved()
    }

    fn can_emit_details(&self) -> bool {
        self.0.can_emit_details()
    }

    fn has_toggled_boss_only_damage(&self) -> bool {
        self.0.has_toggled_boss_only_damage()
    }
}

impl EventManagerInner {
    fn new(app_handle: AppHandle) -> Arc<Self> {
        let inner = Self {
            app_handle: app_handle.clone(),
            subscriptions: Mutex::new(vec![]),
            reset: AtomicBool::new(false),
            save: AtomicBool::new(false),
            pause: AtomicBool::new(false),
            boss_only_damage: AtomicBool::new(false),
            emit_details: AtomicBool::new(false),
        };

        let listener = Arc::new(inner);

        let mut subscriptions = vec![];
        subscriptions.push(app_handle.listen_any("reset-request", Self::on_reset(listener.clone())));
        subscriptions.push(app_handle.listen_any("save-request", Self::on_save(listener.clone())));
        subscriptions.push(app_handle.listen_any("pause-request", Self::on_pause(listener.clone())));
        subscriptions.push(app_handle.listen_any(
            "boss-only-damage-request",
            Self::on_boss_only_damage(listener.clone()),
        ));
        subscriptions.push(app_handle.listen_any(
            "emit-details-request",
            Self::on_emit_details(listener.clone()),
        ));

        *listener.subscriptions.lock().unwrap() = subscriptions;

        listener
    }

    fn on_reset(context: Arc<Self>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            context.reset.store(true, Ordering::Relaxed);
            context.app_handle.emit("reset-encounter", "").unwrap();
        }
    }

    fn on_save(context: Arc<Self>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            context.save.store(true, Ordering::Relaxed);
            context.app_handle.emit("save-encounter", "").unwrap();
        }
    }

    fn on_pause(context: Arc<Self>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            let prev = context.pause.fetch_xor(true, Ordering::Relaxed);

            if prev {
                info!("unpausing meter");
            } else {
                info!("pausing meter");
            }

            context.app_handle.emit("pause-encounter", "").unwrap();
        }
    }

    fn on_boss_only_damage(context: Arc<Self>) -> impl Fn(Event) + Send + 'static {
        move |event: Event| {
            let bod = event.payload();
            
            if bod == "true" {
                context.boss_only_damage.store(true, Ordering::Relaxed);
                info!("boss only damage enabled")
            } else {
                context.boss_only_damage.store(false, Ordering::Relaxed);
                info!("boss only damage disabled")
            }
        }
    }

    fn on_emit_details(context: Arc<Self>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            let prev = context.emit_details.fetch_xor(true, Ordering::Relaxed);

            if prev {
                info!("stopped sending details");
            } else {
                info!("sending details");
            }
        }
    }

    fn has_reset(&self) -> bool {
        let value = self.reset.load(Ordering::Relaxed);
        if value {
            self.reset.store(false, Ordering::Relaxed);
        }
        value
    }

    fn has_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    fn has_saved(&self) -> bool {
        let value = self.save.load(Ordering::Relaxed);
        if value {
            self.save.store(false, Ordering::Relaxed);
        }
        value
    }

    fn can_emit_details(&self) -> bool {
        self.emit_details.load(Ordering::Relaxed)
    }

    fn has_toggled_boss_only_damage(&self) -> bool {
        self.boss_only_damage.load(Ordering::Relaxed)
    }
}

impl TauriEventManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let inner = EventManagerInner::new(app_handle);
        Self(inner)
    }
}

impl Drop for EventManagerInner {
    fn drop(&mut self) {
        for subscription in self.subscriptions.lock().unwrap().drain(..) {
            self.app_handle.unlisten(subscription);
        }
    }
}