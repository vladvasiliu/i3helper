mod i3;

use crate::i3::I3Manager;
use anyhow::{anyhow, Context, Error, Result};
use log::{error, info, LevelFilter, Log};
use std::env::args;
use std::io::Write;
use tokio::net::UnixListener;
use xdg::BaseDirectories;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if systemd_journal_logger::connected_to_journal() {
        systemd_journal_logger::JournalLog::default()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))]);
    } else {
        log::set_logger(&SimpleLogger).unwrap();
    }
    let args = args().collect::<Vec<String>>();
    if args.len() > 2 {
        return Err(anyhow!("Unknown arguments provided"));
    }
    let level = match args.get(1) {
        Some(argument) if argument.eq_ignore_ascii_case("debug") => LevelFilter::Debug,
        Some(argument) => return Err(anyhow!("Unknown argument `{}`", argument)),
        None => LevelFilter::Info,
    };

    log::set_max_level(level);
    info!("Starting i3helper version {}", env!("CARGO_PKG_VERSION"));

    let sock_path = BaseDirectories::new()
        .context("failed to read XDG directories")?
        .place_runtime_file("i3helperd.sock")
        .context("failed to create socket file")?;

    if let Err(err) = (|| async {
        let listener = UnixListener::bind(&sock_path)?;
        info!(
            "Listening for unix domain socket connections at `{}`",
            listener
                .local_addr()?
                .as_pathname()
                .expect("Can't convert local socket to path name")
                .display()
        );

        let mut i3_manager = I3Manager::new(listener).await?;
        i3_manager.run().await?;
        Ok::<(), Error>(())
    })()
    .await
    {
        error!("Something went wrong: {}", err);
    }

    std::fs::remove_file(sock_path)?;

    Ok(())

    // loop {
    //     match listener.accept().await {
    //         Err(e) => {
    //             error!("Connection failed: {}", e);
    //             return Err(e.into());
    //         }
    //         Ok((stream, _addr)) => {
    //             let stream_reader = BufReader::new(stream);
    //             let mut lines = stream_reader.lines();
    //
    //             while let Ok(l) = lines.next_line().await {
    //                 println!("{:?}", l);
    //             }
    //         }
    //     }
    // }
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = writeln!(std::io::stderr(), "{}", record.args());
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}
