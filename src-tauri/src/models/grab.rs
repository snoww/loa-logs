use super::{DamageStats, Encounter, IncapacitatedEvent, IncapacitationEventType};

#[derive(Debug, Default, Clone)]
pub(crate) struct GrabState {
    sequence: u16,
    pub(super) pending_detach: bool,
    movements: Vec<GrabMovement>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct GrabMovement {
    pub timestamp: i64,
    pub sequence: u16,
    pub move_type: u8,
    pub move_time: f32,
    pub freeze_time: f32,
    pub down_time: f32,
    pub stand_up_time: f32,
}

impl DamageStats {
    pub(crate) fn detach_grab(&mut self, timestamp: i64, immediate: bool) {
        self.advance_grab_movements(timestamp);
        if immediate {
            self.end_grab(timestamp);
        } else {
            self.grab_state.pending_detach = self
                .incapacitations
                .iter()
                .any(|event| event.grab_source_id.is_some());
        }
    }

    // Only a real start packet completes deferred detach. Cast and damage
    // notifications also enter the skill statistics path, so they cannot use this.
    pub(crate) fn start_grabbed_skill(&mut self, timestamp: i64) {
        self.advance_grab_movements(timestamp);
        if self.grab_state.pending_detach {
            self.end_grab(timestamp);
        }
        self.clip_paralysis(timestamp);
    }

    pub(crate) fn reset_grab(&mut self, timestamp: i64) {
        self.advance_grab_movements(timestamp);
        self.end_grab(timestamp);
        self.clip_abnormal_moves(timestamp);
        self.grab_state = GrabState::default();
    }

    pub(crate) fn observe_grab_movement(&mut self, movement: GrabMovement, received_at: i64) {
        self.advance_grab_movements(received_at);
        if movement.timestamp <= received_at {
            self.apply_grab_movement(movement);
        } else {
            // Equal application times retain packet arrival order.
            let index = self
                .grab_state
                .movements
                .partition_point(|queued| queued.timestamp <= movement.timestamp);
            self.grab_state.movements.insert(index, movement);
        }
    }

    pub(crate) fn advance_grab_movements(&mut self, timestamp: i64) {
        let count = self
            .grab_state
            .movements
            .partition_point(|movement| movement.timestamp <= timestamp);
        let due = self.grab_state.movements.drain(..count).collect::<Vec<_>>();
        for movement in due {
            self.apply_grab_movement(movement);
        }
    }

    fn has_active_grab(&self) -> bool {
        self.incapacitations
            .iter()
            .any(|event| event.grab_source_id.is_some())
    }

    fn apply_grab_movement(&mut self, movement: GrabMovement) {
        // Native unsigned ordering admits equality. Zero bypasses the comparison
        // and replaces the remembered sequence, including on delayed application.
        if movement.sequence != 0 && movement.sequence < self.grab_state.sequence {
            return;
        }
        self.grab_state.sequence = movement.sequence;
        if movement.move_type == 0 {
            // MoveType::STOP cancels active abnormal movement (native vfunc+2960).
            self.clip_abnormal_moves(movement.timestamp);
            return;
        }
        if matches!(movement.move_type, 2..=12) {
            // Native EFACT_BEHIT replacement movements (STAND_UP, pushes with zero down_time,
            // SWAY, FREEZE_*) replace any active paralysis even when no new timed interval starts.
            self.clip_paralysis(movement.timestamp);
        }
        if matches!(movement.move_type, 2..=8 | 10 | 12) {
            self.end_grab(movement.timestamp);
        }
        if movement.down_time > 0.0 {
            self.clip_abnormal_moves(movement.timestamp);
            self.incapacitations.push(IncapacitatedEvent {
                timestamp: movement.timestamp,
                duration: ((movement.move_time
                    + movement.freeze_time
                    + movement.down_time
                    + movement.stand_up_time)
                    * 1000.0) as i64,
                event_type: IncapacitationEventType::FALL_DOWN,
                grab_source_id: None,
            });
        } else if movement.move_type == 4 {
            // MoveType::SWAY (paralysis): native total duration is move_time + freeze_time
            // (verified in sub_1424939E0 / sub_14247D000 / sub_14230DC60).
            let seconds = movement.move_time.max(0.0) + movement.freeze_time.max(0.0);
            if seconds.is_finite() && seconds > 0.0 {
                let duration = (seconds * 1000.0) as i64;
                if duration > 0 {
                    self.clip_abnormal_moves(movement.timestamp);
                    self.incapacitations.push(IncapacitatedEvent {
                        timestamp: movement.timestamp,
                        duration,
                        event_type: IncapacitationEventType::PARALYSIS,
                        grab_source_id: None,
                    });
                }
            }
        } else if matches!(movement.move_type, 9 | 11) && !self.has_active_grab() {
            // MoveType::FREEZE_STRONG (9) and FREEZE_COUNTER (11): non-knockdown hit-freezes.
            // Native sub_143C44770 passes moveTime and standUpTime (total freeze duration);
            // skip while a grab is active so GRAP_FREEZE_STRONG is not double-labelled as paralysis.
            let seconds = movement
                .move_time
                .max(0.0)
                .max(movement.stand_up_time.max(0.0));
            if seconds.is_finite() && seconds > 0.0 {
                let duration = (seconds * 1000.0) as i64;
                if duration > 0 {
                    self.clip_abnormal_moves(movement.timestamp);
                    self.incapacitations.push(IncapacitatedEvent {
                        timestamp: movement.timestamp,
                        duration,
                        event_type: IncapacitationEventType::PARALYSIS,
                        grab_source_id: None,
                    });
                }
            }
        }
    }

    pub(crate) fn clip_knockdowns(&mut self, timestamp: i64) {
        for event in &mut self.incapacitations {
            if event.event_type == IncapacitationEventType::FALL_DOWN
                && event.timestamp + event.duration > timestamp
            {
                event.duration = (timestamp - event.timestamp).max(0);
            }
        }
    }

    pub(crate) fn clip_paralysis(&mut self, timestamp: i64) {
        for event in &mut self.incapacitations {
            if event.event_type == IncapacitationEventType::PARALYSIS
                && event.timestamp + event.duration > timestamp
            {
                event.duration = (timestamp - event.timestamp).max(0);
            }
        }
    }

    pub(crate) fn clip_abnormal_moves(&mut self, timestamp: i64) {
        for event in &mut self.incapacitations {
            if matches!(
                event.event_type,
                IncapacitationEventType::FALL_DOWN | IncapacitationEventType::PARALYSIS
            ) && event.timestamp + event.duration > timestamp
            {
                event.duration = (timestamp - event.timestamp).max(0);
            }
        }
    }
}

impl Encounter {
    pub(crate) fn advance_grab_movements(&mut self, timestamp: i64) {
        for entity in self
            .entities
            .values_mut()
            .chain(self.current_boss.iter_mut())
        {
            entity.damage_stats.advance_grab_movements(timestamp);
        }
    }

    pub(crate) fn remove_grab_actor(&mut self, actor_id: u64, timestamp: i64) {
        for entity in self.entities.values_mut() {
            entity.damage_stats.advance_grab_movements(timestamp);
            if entity.id == actor_id {
                entity.damage_stats.reset_grab(timestamp);
            } else if entity
                .damage_stats
                .incapacitations
                .iter()
                .any(|event| event.grab_source_id == Some(actor_id))
            {
                entity.damage_stats.end_grab(timestamp);
            }
        }
    }
}
