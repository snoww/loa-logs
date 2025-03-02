use tauri::{generate_handler, Invoke};

mod window;
mod encounter;
mod misc;

pub fn generate_handlers() -> Box<dyn Fn(Invoke) + Send + Sync + 'static> {
    Box::new(generate_handler![
        encounter::load_encounters_preview,
        encounter::load_encounter,
        encounter::get_encounter_count,
        encounter::open_most_recent_encounter,
        encounter::delete_encounter,
        encounter::delete_encounters,
        window::toggle_meter_window,
        window::toggle_logs_window,
        misc::open_url,
        misc::save_settings,
        misc::get_settings,
        misc::open_folder,
        misc::open_db_path,
        encounter::delete_encounters_below_min_duration,
        misc::get_db_info,
        misc::disable_blur,
        misc::enable_blur,
        misc::write_log,
        encounter::toggle_encounter_favorite,
        encounter::delete_all_encounters,
        encounter::delete_all_uncleared_encounters,
        misc::enable_aot,
        misc::disable_aot,
        misc::set_clickthrough,
        misc::optimize_database,
        misc::check_start_on_boot,
        misc::set_start_on_boot,
        misc::check_loa_running,
        misc::start_loa_process,
        encounter::get_sync_candidates,
        encounter::sync,
        misc::remove_driver,
        misc::unload_driver,
    ])
}