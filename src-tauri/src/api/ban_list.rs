use crate::live::debug_print;
use hashbrown::HashSet;
use log::{info, warn};
use reqwest::Client;
use std::time::{Duration, Instant};

const API_URL: &str = "https://snow.xyz/loa-logs/bans.json";

/// character_ids as strings
const FALLBACK_BAN_LIST: &[&str] = &[
    "53572164",
    "40872374",
    "41036",
    "58471929",
    "63777482",
    "55020412",
    "100000009252956",
    "100000006386072",
    "28771077",
    "200000078483801",
    "56128513",
    "52410",
    "53030",
    "15814483",
    "200000078429380",
    "18360071",
    "51887",
    "200000055057950",
    "200000000290673",
    "200000053138501",
    "200000077788602",
    "200000000324781",
    "200000006066343",
    "200000074874868",
    "200000067672505",
    "200000000211516",
    "200000072857264",
    "200000057178278",
    "200000000304676",
    "200000047288504",
    "200000056712487",
    "200000058330360",
    "477134",
    "51796805",
    "100000012734532",
    "8889574",
    "66450531",
    "100000006416740",
    "9434747",
    "8889398",
    "9434881",
    "55027373",
    "100000011985747",
    "840188",
    "32653333",
    "17580231",
    "9434509",
    "4295596",
    "58562896",
    "54323606",
    "34834881",
    "2652002",
    "58566519",
    "200000078474978",
    "44929836",
    "57522810",
    "150198",
    "3673887",
    "200000078469430",
    "36511544",
    "55024934",
    "100000008564927",
    "100000006526696",
    "100000007538511",
    "63999705",
    "16979324",
    "66578929",
    "11694067",
    "8061745",
    "15978706",
    "22267113",
    "26550707",
    "37977865",
    "28531420",
    "100000005381644",
    "62816048",
    "100000011837814",
    "56045870",
    "3375220",
    "59059910",
    "1441579",
    "28466934",
    "45561135",
    "200000078474510",
    "100000011903700",
    "100000008569427",
    "11341307",
    "100000006872536",
    "54031263",
    "66510529",
    "4727476",
    "55041336",
    "56129480",
    "16151256",
    "55275029",
    "55264520",
    "16376348",
    "38682959",
    "200000078588046",
    "13475473",
    "62965343",
    "100000011886706",
    "4237040",
    "4909017",
    "61047",
    "4741223",
    "16439459",
    "66509386",
    "7980918",
    "56408590",
    "100000006507140",
    "6601922",
    "5619248",
    "10111",
    "59056092",
    "66225133",
    "3178663",
    "10517",
    "100000011886225",
    "32369758",
    "200000078473729",
    "14530630",
    "10230",
    "100000007203426",
    "54021898",
    "55032959",
    "100000006500110",
    "16317230",
    "14530344",
    "34499211",
    "37920998",
    "54176708",
    "16950",
    "46627336",
    "35647457",
    "47683782",
    "31072123",
    "10294",
    "36219719",
    "1658866",
    "32466072",
    "58534093",
    "200000078181567",
    "100000008507997",
    "100000007070704",
    "16404967",
    "100000011913517",
    "66450523",
    "24293429",
    "44931815",
    "100000006472869",
    "14348866",
    "20029916",
    "100000008472360",
    "56134429",
    "32466644",
    "100000011885435",
];

pub struct BanList {
    client: Client,
    ids: HashSet<u64>,
    last_fetch: Option<Instant>,
    fetch_interval: Duration,
}

impl BanList {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            ids: parse_ids(FALLBACK_BAN_LIST.iter().copied()),
            last_fetch: None,
            fetch_interval: Duration::from_secs(60 * 60),
        }
    }

    /// refresh the ban list from the API, falling back to list if unreachable.
    pub fn refresh(&mut self) {
        if self
            .last_fetch
            .is_some_and(|t| t.elapsed() < self.fetch_interval)
        {
            return;
        }

        let client = self.client.clone();

        let result = tokio::runtime::Handle::current().block_on(async move {
            client
                .get(API_URL)
                .timeout(Duration::from_secs(5))
                .send()
                .await?
                .json::<Vec<String>>()
                .await
        });

        self.last_fetch = Some(Instant::now());

        match result {
            Ok(strings) => {
                debug_print(format_args!("fetched {} ids from ban list", strings.len()));
                let mut ids = parse_ids(strings.iter().map(|s| s.as_str()));
                ids.extend(parse_ids(FALLBACK_BAN_LIST.iter().copied()));
                self.ids = ids;
            }
            Err(_) => {
                debug_print(format_args!("failed to fetch ban list"));
            }
        }
    }

    pub fn is_banned(&self, character_id: u64) -> bool {
        #[cfg(debug_assertions)]
        {
            return false;
        }
        self.ids.contains(&character_id)
    }
}

fn parse_ids<'a>(strings: impl Iterator<Item = &'a str>) -> HashSet<u64> {
    strings
        .filter_map(|s| {
            s.parse::<u64>()
                .map_err(|e| {
                    debug_print(format_args!("invalid character id in ban list: {s:?}: {e}"))
                })
                .ok()
        })
        .collect()
}
