use hashbrown::{HashMap, HashSet};
use reqwest::Client;
use std::sync::{
    Arc, LazyLock, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

const API_URL: &str = "https://api.snow.xyz/bans";
const FALLBACK_REGION: &str = "EUC";

const FALLBACK_BAN_LIST: &[u64] = &[
    15456655,
    6457750,
    100000015408805,
    100000016025660,
    100000019994545,
    47162109,
    100000015875932,
    15464604,
    100000013350804,
    100000017758691,
    18104664,
];

pub struct BanList {
    client: Client,
    ids_by_region: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    region: Option<String>,
    last_fetch: Option<Instant>,
    fetch_interval: Duration,
    fetch_in_progress: Arc<AtomicBool>,
}

impl BanList {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            ids_by_region: Arc::new(RwLock::new(FALLBACK_IDS_BY_REGION.clone())),
            region: None,
            last_fetch: None,
            fetch_interval: Duration::from_secs(60 * 15),
            fetch_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_region(&mut self, region: Option<String>) {
        self.region = region;
    }

    /// Start refreshing the ban list from the API, falling back to list if unreachable.
    /// The live packet loop must not wait on this network request.
    pub fn refresh(&mut self) {
        if self
            .last_fetch
            .is_some_and(|t| t.elapsed() < self.fetch_interval)
        {
            return;
        }

        self.last_fetch = Some(Instant::now());

        if self.fetch_in_progress.swap(true, Ordering::AcqRel) {
            return;
        }

        let client = self.client.clone();
        let ids_by_region = self.ids_by_region.clone();
        let fetch_in_progress = self.fetch_in_progress.clone();

        tokio::runtime::Handle::current().spawn(async move {
            let result = async {
                client
                    .get(API_URL)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await?
                    .json::<HashMap<String, Vec<u64>>>()
                    .await
            }
            .await;

            let next_ids = match result {
                Ok(by_region) => {
                    let total: usize = by_region.values().map(|v| v.len()).sum();
                    debug_print!("fetched {total} ids from ban list");
                    by_region
                        .into_iter()
                        .map(|(region, ids)| (region, ids.into_iter().collect()))
                        .collect()
                }
                Err(e) => {
                    debug_print!("failed to fetch ban list, using fallback: {e}");
                    FALLBACK_IDS_BY_REGION.clone()
                }
            };

            if let Ok(mut current_ids) = ids_by_region.write() {
                *current_ids = next_ids;
            }

            fetch_in_progress.store(false, Ordering::Release);
        });
    }

    pub fn is_banned(&self, character_id: u64) -> bool {
        let region = self.region.as_deref().unwrap_or(FALLBACK_REGION);
        let Ok(ids_by_region) = self.ids_by_region.read() else {
            return false;
        };
        ids_by_region
            .get(region)
            .is_some_and(|ids| ids.contains(&character_id))
    }
}

static FALLBACK_IDS_BY_REGION: LazyLock<HashMap<String, HashSet<u64>>> = LazyLock::new(|| {
    let mut ids_by_region = HashMap::new();
    ids_by_region.insert(
        FALLBACK_REGION.to_string(),
        FALLBACK_BAN_LIST.iter().copied().collect(),
    );
    ids_by_region
});
