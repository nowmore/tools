use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::never,
};

pub fn init_log() -> WorkerGuard {
    let time_fmt = time::macros::format_description!(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second].[subsecond digits:3]"
    );
    let time_offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time_offset, time_fmt);
    let (non_blocking, guard) = NonBlocking::new(never("I:/programs/bangumi", "bgm.log"));

    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_timer(timer)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    guard
}
