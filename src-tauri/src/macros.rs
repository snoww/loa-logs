macro_rules! debug_print {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            log::info!($($arg)*);
        }
        #[cfg(not(debug_assertions))]
        {
            ()
        }
    }};
}
