use std::str::FromStr;

use anyhow::{Ok, Result};
use hashbrown::HashMap;
use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter};

use crate::{
    database::queries::*,
    models::{player::Specialisation, *},
};
pub struct StatsRepository(r2d2::Pool<SqliteConnectionManager>);

impl StatsRepository {
    pub fn new(connection: r2d2::Pool<SqliteConnectionManager>) -> Self {
        Self(connection)
    }

    pub fn get_stats(&self, args: GetStatsArgs) -> Result<GetStatsResult> {
         let GetStatsArgs {
            date_from,
            date_to,
        } = args;

        let connection = self.0.get()?;
        let params = params![date_from.timestamp_millis(), date_to.timestamp_millis()];
        let mut statement = connection.prepare(SELECT_FROM_ENCOUNTER_STATS)?;
        let iter  = statement.query_map(params_from_iter(params), map_row)?;
        let result = GetStatsResult {
            items: calculate_stats(iter)
        };

        Ok(result)
    }
}

pub fn calculate_stats<I>(iter: I) -> Vec<RaidStats>
where
    I: IntoIterator<Item = rusqlite::Result<RaidMetric>>,
{
    let mut agg: HashMap<RaidType, Vec<RaidMetric>> = HashMap::new();

    for item in iter.into_iter().flatten() {
        if item.kind == RaidType::Unknown {
            continue;
        }

        agg.entry(item.kind)
            .or_default()
            .push(item);
    }

    let metrics = agg.into_iter().map(|(raid_type, instances)| {
        let count = instances.len() as u32;

        let mut total_dps = 0_i64;
        let mut total_ap = 0_f32;
        let mut total_brand = 0_f32;
        let mut total_id = 0_f32;
        let mut total_h = 0_f32;
        let mut dps_count = 0_u32;
        let mut support_count = 0_u32;

        for r in &instances {
            if r.played_as_support {
                total_ap += r.support_ap;
                total_brand += r.support_brand;
                total_id += r.support_identity;
                total_h += r.support_hyper;
                support_count += 1;
            } else {
                total_dps += r.dps;
                dps_count += 1;
            }
        }

        let dps = if dps_count > 0 {
            let raw = total_dps as f64 / dps_count as f64;
            let formatted = abbreviate_number(raw, 2);
            Some(Unit { raw: raw as i64, formatted })
        } else {
            None
        };

        let uptimes = if support_count > 0 {
            let sc = support_count as f32;
            Some((
                format!("{:.2}", total_ap / sc),
                format!("{:.2}", total_brand / sc),
                format!("{:.2}", total_id / sc),
                format!("{:.2}", total_h / sc)
            ))
        } else {
            None
        };

        let is_final_gate = raid_type.is_final_gate();
        let is_guardian_raid = raid_type.is_guardian_raid();
        let order = raid_type.order();
        let name = raid_type.as_ref().to_string();

        RaidStats {
            name,
            order,
            raid_type,
            count,
            dps,
            uptimes,
            instances,
            is_final_gate,
            is_guardian_raid
        }
        
    })
    .collect::<Vec<_>>();

    metrics
}

pub fn map_row(row: &rusqlite::Row) -> rusqlite::Result<RaidMetric> {

    let current_boss: String = row.get("current_boss")?;
    let kind: RaidType = current_boss.into();
    let dps = row.get("dps")?;
    let support_ap = row.get("support_ap")?;
    let support_brand = row.get("support_brand")?;
    let support_identity = row.get("support_identity")?;
    let support_hyper = row.get("support_hyper")?;
    let spec: String = row.get("spec")?;
    let spec = Specialisation::from_str(&spec).unwrap_or_default();
    let played_as_support = spec.is_support();

    let record = RaidMetric {
        kind,
        played_as_support,
        dps,
        support_ap,
        support_brand,
        support_hyper,
        support_identity
    };

    std::result::Result::Ok(record)
}

pub fn abbreviate_number(n: f64, round: usize) -> String {
    let fmt = |v: f64, suffix: &str| -> String {
        format!("{:.*}{}", round, v, suffix)
    };

    if n >= 1e3 && n < 1e6 {
        fmt(n / 1e3, "k")
    } else if n >= 1e6 && n < 1e9 {
        fmt(n / 1e6, "m")
    } else if n >= 1e9 && n < 1e12 {
        fmt(n / 1e9, "b")
    } else if n >= 1e12 {
        fmt(n / 1e12, "t")
    } else {
        format!("{:.0}", n)
    }
}