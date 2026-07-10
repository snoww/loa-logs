use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use log::*;
use tauri::{AppHandle, Emitter, Event, EventId, Listener};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Command {
    Reset,
    Save,
}

pub struct EventManager {
    app_handle: AppHandle,
    subscriptions: Mutex<Vec<EventId>>,
    command_tx: UnboundedSender<Command>,
    boss_only_damage: AtomicBool,
    emit_details: AtomicBool,
}

impl EventManager {
    pub fn new(app_handle: AppHandle, command_tx: UnboundedSender<Command>) -> Arc<Self> {
        let boss_only_damage = AtomicBool::new(true);
        let emit_details = AtomicBool::new(false);

        let listener = Arc::new(Self {
            app_handle: app_handle.clone(),
            subscriptions: Mutex::new(vec![]),
            command_tx,
            boss_only_damage,
            emit_details,
        });

        let mut subscriptions = vec![];
        let id = app_handle.listen_any("reset-request", Self::on_reset(listener.clone()));
        subscriptions.push(id);

        let id = app_handle.listen_any("save-request", Self::on_save(listener.clone()));
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
            if context.command_tx.send(Command::Reset).is_err() {
                warn!("could not queue meter reset");
                return;
            }
            info!("resetting meter");
            context.app_handle.emit("reset-encounter", "").unwrap();
        }
    }

    fn on_save(context: Arc<EventManager>) -> impl Fn(Event) + Send + 'static {
        move |_| {
            if context.command_tx.send(Command::Save).is_err() {
                warn!("could not queue manual encounter save");
            }
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
