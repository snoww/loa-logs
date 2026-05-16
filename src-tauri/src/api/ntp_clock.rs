use chrono::Utc;
use log::{debug, warn};
use rsntp::{Config, SntpClient};
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const NTP_SERVERS: &[&str] = &["time.windows.com", "pool.ntp.org", "time.cloudflare.com"];
const REQUEST_TIMEOUT: Duration = Duration::from_millis(750);
const MAX_SAMPLE_RTT: Duration = Duration::from_millis(1_000);
const REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);
const RETRY_INTERVAL: Duration = Duration::from_secs(15);
const MAX_CACHE_AGE_MS: u64 = 15 * 60 * 1_000;
const MAX_WALL_CLOCK_DRIFT_MS: i64 = 2_000;

#[derive(Debug)]
pub(crate) struct NtpClock {
    state: Arc<Mutex<NtpClockState>>,
    stop: Arc<AtomicBool>,
}

#[derive(Debug, Default)]
struct NtpClockState {
    sample: Option<NtpSample>,
    last_error: Option<String>,
    logged_unavailable: bool,
}

#[derive(Debug, Clone)]
struct NtpSample {
    offset_ms: i64,
    sampled_at: Instant,
    sampled_wall_ms: i64,
    rtt: Duration,
    server: &'static str,
}

impl NtpClock {
    pub(crate) fn start() -> Self {
        let state = Arc::new(Mutex::new(NtpClockState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        spawn_refresh_worker(state.clone(), stop.clone());

        Self { state, stop }
    }

    pub(crate) fn timestamp_for_event(
        &self,
        local_timestamp_ms: i64,
        event_instant: Instant,
    ) -> Option<i64> {
        let mut state = lock_state(&self.state);
        let Some(sample) = fresh_sample_for_event(&state, event_instant) else {
            log_unavailable_once(&mut state, "no fresh cached NTP offset");
            return None;
        };

        let Some(instant_local_ms) = local_time_for_instant(&sample, event_instant) else {
            log_unavailable_once(&mut state, "NTP instant conversion overflowed i64");
            return None;
        };
        let wall_clock_delta_ms = instant_local_ms.saturating_sub(local_timestamp_ms);
        let timestamp_ms = if wall_clock_delta_ms.unsigned_abs() <= MAX_WALL_CLOCK_DRIFT_MS as u64 {
            local_timestamp_ms
        } else {
            debug!(
                "using monotonic NTP conversion after wall-clock discontinuity: delta={}ms",
                wall_clock_delta_ms
            );
            instant_local_ms
        };

        timestamp_ms.checked_add(sample.offset_ms).or_else(|| {
            log_unavailable_once(&mut state, "NTP timestamp overflowed i64");
            None
        })
    }
}

fn local_time_for_instant(sample: &NtpSample, instant: Instant) -> Option<i64> {
    duration_between_instants(instant, sample.sampled_at)
        .and_then(|delta_ms| sample.sampled_wall_ms.checked_add(delta_ms))
}

impl Drop for NtpClock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn spawn_refresh_worker(state: Arc<Mutex<NtpClockState>>, stop: Arc<AtomicBool>) {
    let worker_state = state.clone();
    let worker_stop = stop.clone();
    let spawn_result = thread::Builder::new()
        .name("ntp-clock-refresh".to_string())
        .spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let refreshed = refresh_once(&worker_state);
                sleep_or_stop(
                    &worker_stop,
                    if refreshed {
                        REFRESH_INTERVAL
                    } else {
                        RETRY_INTERVAL
                    },
                );
            }
        });

    if let Err(err) = spawn_result {
        let mut state = lock_state(&state);
        state.last_error = Some(format!("failed to spawn NTP refresh worker: {err}"));
        warn!("failed to spawn NTP refresh worker: {err}");
    }
}

fn refresh_once(state: &Arc<Mutex<NtpClockState>>) -> bool {
    let client = SntpClient::with_config(Config::default().timeout(REQUEST_TIMEOUT));
    let mut samples = Vec::new();
    let mut errors = Vec::new();

    for &server in NTP_SERVERS {
        match query_server(&client, server) {
            Ok(sample) => samples.push(sample),
            Err(err) => errors.push(format!("{server}: {err}")),
        }
    }

    let Some(best_sample) = samples.into_iter().min_by_key(|sample| sample.rtt) else {
        let error = if errors.is_empty() {
            "no NTP servers configured".to_string()
        } else {
            errors.join("; ")
        };
        let mut state = lock_state(state);
        state.last_error = Some(error.clone());
        debug!("NTP refresh failed: {error}");
        return false;
    };

    let mut state = lock_state(state);
    debug!(
        "NTP offset refreshed from {}: offset={}ms rtt={:?}",
        best_sample.server, best_sample.offset_ms, best_sample.rtt
    );
    state.sample = Some(best_sample);
    state.last_error = None;
    state.logged_unavailable = false;
    true
}

fn query_server(client: &SntpClient, server: &'static str) -> Result<NtpSample, String> {
    let result = client
        .synchronize(server)
        .map_err(|err| format!("synchronize failed: {err}"))?;

    let rtt = result
        .round_trip_delay()
        .abs_as_std_duration()
        .map_err(|err| format!("invalid round-trip delay: {err}"))?;
    if rtt > MAX_SAMPLE_RTT {
        return Err(format!(
            "round-trip delay {rtt:?} exceeded {MAX_SAMPLE_RTT:?}"
        ));
    }

    let offset = result
        .clock_offset()
        .into_chrono_duration()
        .map_err(|err| format!("invalid clock offset: {err}"))?;

    Ok(NtpSample {
        offset_ms: offset.num_milliseconds(),
        sampled_at: Instant::now(),
        sampled_wall_ms: Utc::now().timestamp_millis(),
        rtt,
        server,
    })
}

fn fresh_sample_for_event(state: &NtpClockState, event_instant: Instant) -> Option<NtpSample> {
    let sample = state.sample.clone()?;
    let sample_age_ms = duration_between_instants(event_instant, sample.sampled_at)?.unsigned_abs();
    (sample_age_ms <= MAX_CACHE_AGE_MS).then_some(sample)
}

fn duration_between_instants(target: Instant, sample: Instant) -> Option<i64> {
    if target >= sample {
        duration_to_i64_ms(target.duration_since(sample))
    } else {
        duration_to_i64_ms(sample.duration_since(target)).and_then(|millis| millis.checked_neg())
    }
}

fn duration_to_i64_ms(duration: Duration) -> Option<i64> {
    i64::try_from(duration.as_millis()).ok()
}

fn log_unavailable_once(state: &mut NtpClockState, reason: &str) {
    if state.logged_unavailable {
        return;
    }

    match &state.last_error {
        Some(last_error) => {
            warn!("NTP timestamp unavailable: {reason}; last refresh error: {last_error}")
        }
        None => warn!("NTP timestamp unavailable: {reason}"),
    }
    state.logged_unavailable = true;
}

fn sleep_or_stop(stop: &AtomicBool, duration: Duration) {
    let started = Instant::now();
    while !stop.load(Ordering::Relaxed) && started.elapsed() < duration {
        let remaining = duration.saturating_sub(started.elapsed());
        thread::sleep(remaining.min(Duration::from_secs(1)));
    }
}

fn lock_state(state: &Arc<Mutex<NtpClockState>>) -> MutexGuard<'_, NtpClockState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
