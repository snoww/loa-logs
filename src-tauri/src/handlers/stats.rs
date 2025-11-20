use super::error::*;
use tauri::{command, State};

use crate::database::{Database, StatsRepository};
use crate::models::*;

#[command]
pub fn get_raid_stats(
    repository: State<StatsRepository>,
    args: GetStatsArgs
) -> Result<GetStatsResult> {

    let result = repository.get_stats(args)?;

    Ok(result)
}