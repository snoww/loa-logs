use std::sync::{
    atomic::{AtomicBool, Ordering}, Arc,
    Mutex,
};

use log::*;
use tauri::{AppHandle, Emitter, Event, EventId, Listener};

pub struct EventManager {
    app_handle: AppHandle,
    subscriptions: Mutex<Vec<EventId>>,
    reset: AtomicBool,
    save: AtomicBool,
    pause: AtomicBool,
    boss_only_damage: AtomicBool,
    emit_details: AtomicBool,
}

impl EventManager {
    pub fn new(app_handle: AppHandle) -> Arc<Self> {
        let reset = AtomicBool::new(false);
        let pause = AtomicBool::new(false);
        let save = AtomicBool::new(false);
        let boss_only_damage = AtomicBool::new(true);
        let emit_details = AtomicBool::new(false);

        let listener = Arc::new(Self {
            app_handle: app_handle.clone(),
            subscriptions: Mutex::new(vec![]),
            reset,
            save,
            pause,
            boss_only_damage,
            emit_details,
        });

        let mut subscriptions = vec![];
        let id = app_handle.listen_any("reset-request", Self::on_reset(listener.clone()));
        subscriptions.push(id);

        let id = app_handle.listen_any("save-request", Self::on_save(listener.clone()));
        subscriptions.push(id);

        let id = app_handle.listen_any("pause-request", Self::on_pause(listener.clone()));
        subscriptions.push(id);

        let id = app_handle.listen_any(
            "boss-only-damage-request",
            Self::on_boss_only_damage(listener.clone()),
        );
        subscriptions.push(id);

        let id = app_handle.listen_any(
            "emit-details-request",
            Self::on_emit_details(listener.clone()),
        );
        subscriptions.push(id);

        *listener.subscriptions.lock().unwrap() = subscriptions;

        listener
    }

    fn on_reset(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            context.reset.store(true, Ordering::Relaxed);
            info!("resetting meter");
            context.app_handle.emit("reset-encounter", "").unwrap();
        }
    }

    fn on_save(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            context.save.store(true, Ordering::Relaxed);
            info!("manual saving encounter");
            context.app_handle.emit("save-encounter", "").unwrap();
        }
    }

    fn on_pause(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
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

    fn on_boss_only_damage(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
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

    fn on_emit_details(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
        move |_event: Event| {
            let prev = context.emit_details.fetch_xor(true, Ordering::Relaxed);

            if prev {
                info!("stopped sending details");
            } else {
                info!("sending details");
            }
        }
    }

    pub fn set_boss_only_damage(&self) {
        self.boss_only_damage.store(true, Ordering::Relaxed);
    }

    pub fn has_reset(&self) -> bool {
        let value = self.reset.load(Ordering::Relaxed);

        if value {
            self.reset.store(false, Ordering::Relaxed);
        }

        value
    }

    pub fn has_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    pub fn has_saved(&self) -> bool {
        let value = self.save.load(Ordering::Relaxed);

        if value {
            self.save.store(false, Ordering::Relaxed);
        }

        value
    }

    pub fn can_emit_details(&self) -> bool {
        self.emit_details.load(Ordering::Relaxed)
    }

    pub fn has_toggled_boss_only_damage(&self) -> bool {
        self.boss_only_damage.load(Ordering::Relaxed)
    }
}

impl Drop for EventManager {
    fn drop(&mut self) {
        for subscription in self.subscriptions.lock().unwrap().drain(..) {
            self.app_handle.unlisten(subscription);
        }
    }
}
