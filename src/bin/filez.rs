use filez::ui::LogRecord;
use std::io;
use clap::{arg, command};

fn main() -> io::Result<()> {
    let matches = command!()
        .arg(arg!([LEFT]))
        .arg(arg!([RIGHT]))
        .get_matches();

    let left = matches.value_of("LEFT").expect("left path not specify");
    let right = matches.value_of("RIGHT").expect("right path not specify");

    let (tx, rx) = std::sync::mpsc::sync_channel(1024);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(move |buf, record| {
            let _ = tx
                .try_send(LogRecord {
                    level: record.level(),
                    target: record.target().to_string(),
                    msg: record.args().to_string(),
                    timestamp: buf.timestamp().to_string(),
                })
                .ok();
            Ok(())
        })
        .init();

    log::info!("hello from main");
    log::debug!("hello from main");
    log::error!("hello from main");

    filez::ui::run(rx, left, right)
}
