use crate::data::SKILL_WINDOW_DATA;
use crate::live::entity_tracker::Entity;
use crate::live::stat_type::StatType;
use crate::models::EntityType;
use hashbrown::HashMap;
use serde::Deserialize;

const EXPECTED_SCHEMA_VERSION: u32 = 1;
const INSTANT_PAYLOAD_THRESHOLD_SECONDS: f64 = 0.02;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillWindowData {
    pub schema_version: u32,
    pub skills: HashMap<u32, SkillWindowEntry>,
    pub stages: Vec<SkillStageWindow>,
}

#[derive(Debug, Deserialize)]
pub struct SkillWindowEntry {
    pub layers: Vec<u32>,
    pub stages: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillStageKind {
    Windup,
    ContinuingPayload,
    FinalPayload,
    Recovery,
    EffectlessTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SingleBurstKind {
    Projectile,
    Trap,
    DirectDamage,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStageWindow {
    pub kind: SkillStageKind,
    #[serde(default)]
    pub last_effect: f64,
    #[serde(default = "default_play_rate")]
    pub rate: f64,
    #[serde(default)]
    pub attack_speed: bool,
    #[serde(default)]
    pub single_burst: Option<SingleBurstKind>,
}

fn default_play_rate() -> f64 {
    1.0
}

#[derive(Debug, Clone)]
pub struct ActiveSkillCast {
    pub skill_id: u32,
    pub layer: u32,
    pub stage: u8,
    pub stage_start_time: f64,
    pub stage_entered_at_ms: i64,
    pub attack_speed: Option<i64>,
    pub visited_payload_stage: bool,
    pub stage_spawned_projectile: bool,
    pub stage_spawned_trap: bool,
    pub stage_dealt_direct_damage: bool,
}

impl SkillStageKind {
    fn is_payload_stage(self) -> bool {
        matches!(self, Self::ContinuingPayload | Self::FinalPayload)
    }
}

impl SkillWindowData {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema_version == EXPECTED_SCHEMA_VERSION,
            "Unsupported SkillWindows.json schemaVersion {}",
            self.schema_version
        );
        for (idx, stage) in self.stages.iter().enumerate() {
            if stage.kind == SkillStageKind::FinalPayload {
                anyhow::ensure!(
                    stage.last_effect.is_finite() && stage.last_effect >= 0.0,
                    "Invalid SkillWindows stage {idx} lastEffect {}",
                    stage.last_effect
                );
                anyhow::ensure!(
                    stage.rate.is_finite() && stage.rate > 0.0,
                    "Invalid SkillWindows stage {idx} rate {}",
                    stage.rate
                );
            }
        }
        for (&skill_id, entry) in &self.skills {
            let total: usize = entry.layers.iter().map(|&count| count as usize).sum();
            anyhow::ensure!(
                total == entry.stages.len(),
                "SkillWindows skill {skill_id} layer sum {total} != stages len {}",
                entry.stages.len()
            );
            for &stage_ref in &entry.stages {
                anyhow::ensure!(
                    stage_ref < self.stages.len(),
                    "SkillWindows skill {skill_id} stage ref {stage_ref} out of bounds"
                );
            }
        }
        Ok(())
    }

    pub fn resolve_stage(
        &self,
        skill_id: u32,
        layer: u32,
        local_stage: u8,
    ) -> Option<&SkillStageWindow> {
        let entry = self.skills.get(&skill_id)?;
        let layer_idx = usize::try_from(layer).ok()?;
        let stage_count = *entry.layers.get(layer_idx)?;
        if u32::from(local_stage) >= stage_count {
            return None;
        }
        let offset = entry.layers[..layer_idx]
            .iter()
            .try_fold(0usize, |acc, &count| acc.checked_add(count as usize))?
            .checked_add(usize::from(local_stage))?;
        let pool_idx = *entry.stages.get(offset)?;
        self.stages.get(pool_idx)
    }

    pub fn single_layer_default(&self, skill_id: u32) -> Option<u32> {
        let entry = self.skills.get(&skill_id)?;
        (entry.layers.len() == 1).then_some(0)
    }

    pub fn should_suppress_cancel(&self, cast: &ActiveSkillCast, timestamp_ms: i64) -> bool {
        let Some(stage) = self.resolve_stage(cast.skill_id, cast.layer, cast.stage) else {
            return false;
        };
        match stage.kind {
            SkillStageKind::Windup | SkillStageKind::ContinuingPayload => false,
            SkillStageKind::Recovery => true,
            SkillStageKind::EffectlessTerminal => cast.visited_payload_stage,
            SkillStageKind::FinalPayload => {
                if stage.last_effect <= INSTANT_PAYLOAD_THRESHOLD_SECONDS {
                    return true;
                }
                match stage.single_burst {
                    Some(SingleBurstKind::Projectile) if cast.stage_spawned_projectile => {
                        return true;
                    }
                    Some(SingleBurstKind::Trap) if cast.stage_spawned_trap => return true,
                    Some(SingleBurstKind::DirectDamage) if cast.stage_dealt_direct_damage => {
                        return true;
                    }
                    _ => {}
                }
                let speed_scale = if stage.attack_speed {
                    match cast.attack_speed {
                        Some(speed) if speed > 0 => speed as f64 / 100.0,
                        _ => 1.0,
                    }
                } else {
                    1.0
                };
                let effective_rate = stage.rate * speed_scale;
                if !effective_rate.is_finite() || effective_rate <= 0.0 {
                    return false;
                }
                let elapsed_real_secs =
                    timestamp_ms.saturating_sub(cast.stage_entered_at_ms) as f64 / 1000.0;
                let stage_time = cast.stage_start_time + elapsed_real_secs * effective_rate;
                stage_time >= stage.last_effect
            }
        }
    }
}

impl Entity {
    pub fn observe_player_skill_start(
        &mut self,
        skill_id: u32,
        layer: u32,
        stage: u8,
        stage_start_time: f64,
        timestamp_ms: i64,
    ) {
        self.observe_player_skill_start_with(
            SKILL_WINDOW_DATA.try_get(),
            skill_id,
            layer,
            stage,
            stage_start_time,
            timestamp_ms,
        );
    }

    pub(crate) fn observe_player_skill_start_with(
        &mut self,
        windows: Option<&SkillWindowData>,
        skill_id: u32,
        layer: u32,
        stage: u8,
        stage_start_time: f64,
        timestamp_ms: i64,
    ) {
        if self.entity_type != EntityType::Player || skill_id == 0 {
            return;
        }
        let start_time = if stage_start_time.is_finite() && stage_start_time > 0.0 {
            stage_start_time
        } else {
            0.0
        };
        let visited_payload_stage = windows
            .and_then(|w| w.resolve_stage(skill_id, layer, stage))
            .is_some_and(|st| st.kind.is_payload_stage());
        self.active_skill_cast = Some(ActiveSkillCast {
            skill_id,
            layer,
            stage,
            stage_start_time: start_time,
            stage_entered_at_ms: timestamp_ms,
            attack_speed: self.stats.get(&(StatType::ATTACK_SPEED as u8)).copied(),
            visited_payload_stage,
            stage_spawned_projectile: false,
            stage_spawned_trap: false,
            stage_dealt_direct_damage: false,
        });
    }

    pub fn observe_player_skill_stage(&mut self, skill_id: u32, stage: u8, timestamp_ms: i64) {
        self.observe_player_skill_stage_with(
            SKILL_WINDOW_DATA.try_get(),
            skill_id,
            stage,
            timestamp_ms,
        );
    }

    pub(crate) fn observe_player_skill_stage_with(
        &mut self,
        windows: Option<&SkillWindowData>,
        skill_id: u32,
        stage: u8,
        timestamp_ms: i64,
    ) {
        if self.entity_type != EntityType::Player || skill_id == 0 {
            return;
        }
        let attack_speed = self.stats.get(&(StatType::ATTACK_SPEED as u8)).copied();
        if let Some(cast) = self
            .active_skill_cast
            .as_mut()
            .filter(|cast| cast.skill_id == skill_id)
        {
            cast.stage = stage;
            cast.stage_start_time = 0.0;
            cast.stage_entered_at_ms = timestamp_ms;
            cast.attack_speed = attack_speed;
            cast.stage_spawned_projectile = false;
            cast.stage_spawned_trap = false;
            cast.stage_dealt_direct_damage = false;
            if windows
                .and_then(|w| w.resolve_stage(skill_id, cast.layer, stage))
                .is_some_and(|st| st.kind.is_payload_stage())
            {
                cast.visited_payload_stage = true;
            }
            return;
        }

        // If the start packet was missed (e.g. joined mid-cast), still bind unambiguous
        // single-layer skills like Doomsday when their release stage packet arrives.
        if let Some(layer) = windows.and_then(|w| w.single_layer_default(skill_id)) {
            let visited_payload_stage = windows
                .and_then(|w| w.resolve_stage(skill_id, layer, stage))
                .is_some_and(|st| st.kind.is_payload_stage());
            self.active_skill_cast = Some(ActiveSkillCast {
                skill_id,
                layer,
                stage,
                stage_start_time: 0.0,
                stage_entered_at_ms: timestamp_ms,
                attack_speed,
                visited_payload_stage,
                stage_spawned_projectile: false,
                stage_spawned_trap: false,
                stage_dealt_direct_damage: false,
            });
        }
    }

    pub fn observe_player_skill_projectile(&mut self, skill_id: u32) {
        if let Some(cast) = self
            .active_skill_cast
            .as_mut()
            .filter(|cast| cast.skill_id == skill_id)
        {
            cast.stage_spawned_projectile = true;
        }
    }

    pub fn observe_player_skill_trap(&mut self, skill_id: u32) {
        if let Some(cast) = self
            .active_skill_cast
            .as_mut()
            .filter(|cast| cast.skill_id == skill_id)
        {
            cast.stage_spawned_trap = true;
        }
    }

    pub fn observe_player_skill_direct_damage(&mut self, skill_id: u32) {
        if let Some(cast) = self
            .active_skill_cast
            .as_mut()
            .filter(|cast| cast.skill_id == skill_id)
        {
            cast.stage_dealt_direct_damage = true;
        }
    }

    pub fn consume_skill_cancel(&mut self, skill_id: u32, timestamp_ms: i64) -> bool {
        self.consume_skill_cancel_with(SKILL_WINDOW_DATA.try_get(), skill_id, timestamp_ms)
    }

    pub(crate) fn consume_skill_cancel_with(
        &mut self,
        windows: Option<&SkillWindowData>,
        skill_id: u32,
        timestamp_ms: i64,
    ) -> bool {
        let Some(cast) = self
            .active_skill_cast
            .take_if(|cast| cast.skill_id == skill_id)
        else {
            return false;
        };
        windows.is_some_and(|w| w.should_suppress_cancel(&cast, timestamp_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_bundled_windows() -> SkillWindowData {
        let windows: SkillWindowData =
            serde_json::from_str(include_str!("../../meter-data/SkillWindows.json"))
                .expect("bundled SkillWindows.json should deserialize");
        windows
            .validate()
            .expect("bundled SkillWindows.json should validate");
        windows
    }

    fn make_player(attack_speed: Option<i64>) -> Entity {
        let mut entity = Entity {
            id: 1,
            entity_type: EntityType::Player,
            name: "Sorc".to_string(),
            class_id: 205,
            ..Default::default()
        };
        if let Some(speed) = attack_speed {
            entity.stats.insert(StatType::ATTACK_SPEED as u8, speed);
        }
        entity
    }

    #[test]
    fn doomsday_interrupted_during_cast_counts_but_post_cast_animation_is_suppressed() {
        let windows = load_bundled_windows();
        let mut player = make_player(Some(120));

        // Cast bar wind-up (stage 0) -> real interrupt.
        player.observe_player_skill_start_with(Some(&windows), 37350, 0, 0, 0.0, 1_000);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 37350, 2_200),
            "interrupting Doomsday during casting stage 0 must count"
        );

        // Igniter cast bar wind-up (stage 1) -> real interrupt.
        player.observe_player_skill_start_with(Some(&windows), 37350, 0, 1, 0.0, 5_000);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 37350, 5_400),
            "interrupting Doomsday during casting stage 1 must count"
        );

        // Finished casting -> stage 2 release animation -> must NOT count as interrupt.
        player.observe_player_skill_start_with(Some(&windows), 37350, 0, 0, 0.0, 10_000);
        player.observe_player_skill_stage_with(Some(&windows), 37350, 2, 11_800);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 37350, 12_100),
            "cancelling Doomsday during post-cast follow-through stage 2 must be suppressed"
        );
    }

    #[test]
    fn blaze_and_rime_arrow_suppress_after_projectile_or_clock_but_count_during_startup() {
        let windows = load_bundled_windows();
        let mut player = make_player(Some(100));

        // Blaze (37200): fires projectile at 0.25s.
        player.observe_player_skill_start_with(Some(&windows), 37200, 0, 0, 0.0, 1_000);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 37200, 1_100),
            "Blaze interrupted at 0.10s before projectile must count"
        );

        // Blaze with projectile already spawned (even before clock says 0.25s, e.g. speed tripod).
        player.observe_player_skill_start_with(Some(&windows), 37200, 0, 0, 0.0, 2_000);
        player.observe_player_skill_projectile(37200);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 37200, 2_180),
            "Blaze cancelled after PKTNewProjectile must be suppressed"
        );

        // Blaze cancelled in recovery tail via clock (0.40s >= 0.25s).
        player.observe_player_skill_start_with(Some(&windows), 37200, 0, 0, 0.0, 3_000);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 37200, 3_400),
            "Blaze cancelled in recovery animation must be suppressed"
        );
    }

    #[test]
    fn soundholic_and_bloody_rush_keep_mid_channel_and_mid_combo_interrupts() {
        let windows = load_bundled_windows();
        let mut player = make_player(Some(100));

        // Soundholic (21120): stage 1 beam ticks up to ~2.80s, stage 2 is pure recovery.
        player.observe_player_skill_start_with(Some(&windows), 21120, 0, 0, 0.0, 1_000);
        player.observe_player_skill_stage_with(Some(&windows), 21120, 1, 1_500);
        player.observe_player_skill_direct_damage(21120);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 21120, 2_500),
            "Soundholic interrupted 1.0s into 2.8s beam must STILL count even after first tick"
        );

        // Soundholic stage 2 pure recovery -> suppressed immediately.
        player.observe_player_skill_start_with(Some(&windows), 21120, 0, 0, 0.0, 5_000);
        player.observe_player_skill_stage_with(Some(&windows), 21120, 1, 5_500);
        player.observe_player_skill_stage_with(Some(&windows), 21120, 2, 8_400);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 21120, 8_600),
            "Soundholic cancelled during post-beam recovery stage 2 must be suppressed"
        );

        // Bloody Rush (16140): multi-hit up to 2.51s at rate 0.9 (~2.79s real at 100% AS).
        player.observe_player_skill_start_with(Some(&windows), 16140, 0, 0, 0.0, 10_000);
        player.observe_player_skill_direct_damage(16140);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 16140, 11_000),
            "Bloody Rush interrupted after early hits must STILL count"
        );
        player.observe_player_skill_start_with(Some(&windows), 16140, 0, 0, 0.0, 20_000);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 16140, 23_000),
            "Bloody Rush cancelled in recovery after final hit must be suppressed"
        );
    }

    #[test]
    fn focused_shot_target_down_and_ancient_spear_handle_multi_stage_semantics() {
        let windows = load_bundled_windows();
        let mut player = make_player(Some(100));

        // Focused Shot (38240): shot 1 (s2) auto-branches to s1/s4 -> s3 finisher.
        player.observe_player_skill_start_with(Some(&windows), 38240, 0, 0, 0.0, 1_000);
        player.observe_player_skill_stage_with(Some(&windows), 38240, 2, 1_900);
        player.observe_player_skill_projectile(38240);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 38240, 2_100),
            "Focused Shot interrupted after shot 1 (s2) before finisher must count"
        );

        // Focused Shot finisher stage 3 after hit -> suppressed.
        player.observe_player_skill_start_with(Some(&windows), 38240, 0, 0, 0.0, 5_000);
        player.observe_player_skill_stage_with(Some(&windows), 38240, 3, 7_000);
        player.observe_player_skill_direct_damage(38240);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 38240, 7_450),
            "Focused Shot cancelled in recovery after finisher shot 3 must be suppressed"
        );

        // Target Down (38070): aiming stage 5 counts; holster stage 3 after shot 3 (s7) is suppressed.
        player.observe_player_skill_start_with(Some(&windows), 38070, 0, 0, 0.0, 10_000);
        player.observe_player_skill_stage_with(Some(&windows), 38070, 2, 11_000);
        player.observe_player_skill_stage_with(Some(&windows), 38070, 5, 11_500);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 38070, 12_000),
            "Target Down interrupted while aiming shot 2 (s5) must count"
        );
        player.observe_player_skill_start_with(Some(&windows), 38070, 0, 0, 0.0, 15_000);
        player.observe_player_skill_stage_with(Some(&windows), 38070, 7, 17_000);
        player.observe_player_skill_stage_with(Some(&windows), 38070, 3, 17_500);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 38070, 17_800),
            "Target Down cancelled during post-shot holster stage 3 must be suppressed"
        );

        // Ancient Spear (20260): s0 -> s2 (no payload visited) must NOT be suppressed.
        player.observe_player_skill_start_with(Some(&windows), 20260, 0, 0, 0.0, 20_000);
        player.observe_player_skill_stage_with(Some(&windows), 20260, 2, 20_500);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 20260, 20_800),
            "Ancient Spear effectless s0 -> s2 without visiting payload stage must not suppress"
        );
    }

    #[test]
    fn perfect_swing_respects_attack_speed_and_direct_damage_confirmation() {
        let windows = load_bundled_windows();
        let mut player = make_player(Some(140));

        // Perfect Swing (18170) stage 4: lastEffect = 1.0449s; at 140% AS -> 0.746s real time.
        player.observe_player_skill_start_with(Some(&windows), 18170, 0, 0, 0.0, 1_000);
        player.observe_player_skill_stage_with(Some(&windows), 18170, 4, 2_500);
        assert!(
            !player.consume_skill_cancel_with(Some(&windows), 18170, 3_000),
            "Perfect Swing interrupted at 0.50s (0.70s scaled < 1.045s) before impact must count"
        );

        // Whiffed Perfect Swing in recovery at 0.90s real (1.26s scaled >= 1.045s) -> suppressed.
        player.observe_player_skill_start_with(Some(&windows), 18170, 0, 0, 0.0, 5_000);
        player.observe_player_skill_stage_with(Some(&windows), 18170, 4, 6_500);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 18170, 7_400),
            "Whiffed Perfect Swing cancelled during hammer recovery must be suppressed"
        );

        // Hit Perfect Swing confirmed by direct damage -> suppressed immediately.
        player.observe_player_skill_start_with(Some(&windows), 18170, 0, 0, 0.0, 10_000);
        player.observe_player_skill_stage_with(Some(&windows), 18170, 4, 11_500);
        player.observe_player_skill_direct_damage(18170);
        assert!(
            player.consume_skill_cancel_with(Some(&windows), 18170, 12_100),
            "Perfect Swing cancelled after landing its single direct hit must be suppressed"
        );
    }
}
