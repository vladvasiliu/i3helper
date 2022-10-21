mod i3;

use crate::i3::I3Manager;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use log::{info, LevelFilter};
use std::env::args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if systemd_journal_logger::connected_to_journal() {
        systemd_journal_logger::init_with_extra_fields(vec![(
            "VERSION",
            env!("CARGO_PKG_VERSION"),
        )])?;
    } else {
        simple_logger::SimpleLogger::new().init()?;
    }

    let args = args().collect::<Vec<String>>();
    if args.len() > 2 {
        return Err(eyre!("Unknown arguments provided"));
    }
    let level = match args.get(1) {
        Some(argument) if argument.eq_ignore_ascii_case("debug") => LevelFilter::Debug,
        Some(argument) => return Err(eyre!("Unknown argument `{}`", argument)),
        None => LevelFilter::Info,
    };

    log::set_max_level(level);
    info!("Starting i3helper version {}", env!("CARGO_PKG_VERSION"));

    let mut i3_manager = I3Manager::new().await?;
    i3_manager.run().await?;
    Ok(())
}
