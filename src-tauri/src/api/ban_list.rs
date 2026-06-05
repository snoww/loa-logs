use hashbrown::{HashMap, HashSet};
use reqwest::Client;
use std::sync::{
    Arc, LazyLock, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

const API_URL: &str = "https://api.snow.xyz/bans";
const FALLBACK_REGION: &str = "NA";

const FALLBACK_BAN_LIST: &[u64] = &[
    53572164,
    40872374,
    41036,
    58471929,
    63777482,
    55020412,
    100000009252956,
    100000006386072,
    28771077,
    200000078483801,
    56128513,
    52410,
    53030,
    15814483,
    200000078429380,
    18360071,
    51887,
    200000055057950,
    200000000290673,
    200000053138501,
    200000077788602,
    200000000324781,
    200000006066343,
    200000074874868,
    200000067672505,
    200000000211516,
    200000072857264,
    200000057178278,
    200000000304676,
    200000047288504,
    200000056712487,
    200000058330360,
    477134,
    51796805,
    100000012734532,
    8889574,
    66450531,
    100000006416740,
    9434747,
    8889398,
    9434881,
    55027373,
    100000011985747,
    840188,
    32653333,
    17580231,
    9434509,
    4295596,
    58562896,
    54323606,
    34834881,
    2652002,
    58566519,
    200000078474978,
    44929836,
    57522810,
    150198,
    3673887,
    200000078469430,
    36511544,
    55024934,
    100000008564927,
    100000006526696,
    100000007538511,
    63999705,
    16979324,
    66578929,
    11694067,
    8061745,
    15978706,
    22267113,
    26550707,
    37977865,
    28531420,
    100000005381644,
    62816048,
    100000011837814,
    56045870,
    3375220,
    59059910,
    1441579,
    28466934,
    45561135,
    200000078474510,
    100000011903700,
    100000008569427,
    11341307,
    100000006872536,
    54031263,
    66510529,
    4727476,
    55041336,
    56129480,
    16151256,
    55275029,
    55264520,
    16376348,
    38682959,
    200000078588046,
    13475473,
    62965343,
    100000011886706,
    4237040,
    4909017,
    61047,
    4741223,
    16439459,
    66509386,
    7980918,
    56408590,
    100000006507140,
    6601922,
    5619248,
    10111,
    59056092,
    66225133,
    3178663,
    10517,
    100000011886225,
    32369758,
    200000078473729,
    14530630,
    10230,
    100000007203426,
    54021898,
    55032959,
    100000006500110,
    16317230,
    14530344,
    34499211,
    37920998,
    54176708,
    16950,
    46627336,
    35647457,
    47683782,
    31072123,
    10294,
    36219719,
    1658866,
    32466072,
    58534093,
    200000078181567,
    100000008507997,
    100000007070704,
    16404967,
    100000011913517,
    66450523,
    24293429,
    44931815,
    100000006472869,
    14348866,
    20029916,
    100000008472360,
    56134429,
    32466644,
    100000011885435,
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
