use anyhow::Context;
use chrono::Utc;
use log::LevelFilter;

pub fn setup() -> anyhow::Result<()> {
    #[cfg(not(debug_assertions))]
    let level = LevelFilter::Info;

    #[cfg(debug_assertions)]
    let level = LevelFilter::Trace;

    println!("level: {:?}", level);

    fern::Dispatch::new()
        .format(|out, message, record| {
            let mut file = record.file().unwrap_or("?").to_string();
            if let Some(line) = record.line() {
                file.push(':');
                file.push_str(&line.to_string());
            }

            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                file,
                message
            ))
        })
        .filter(|rec| rec.target().starts_with("server") || rec.target().starts_with("client"))
        .level(LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .context("failed to setup logging")
}
