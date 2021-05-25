mod i3;

use crate::i3::{focus_listener, I3Manager};
use log::error;

#[tokio::main]
async fn main() {
    setup_logger(log::LevelFilter::Info).unwrap();
    let mut i3_manager = I3Manager::new().await.unwrap();

    let mut join_handles = vec![];
    let focus_listener_tx = i3_manager.get_i3_event_tx();
    join_handles.push(tokio::spawn(
        async move { i3_manager.i3_cmd_sender().await },
    ));
    join_handles.push(tokio::spawn(async {
        focus_listener(focus_listener_tx).await
    }));
    for jh in join_handles.drain(..) {
        if let Err(err) = jh.await {
            error!("{}", err)
        }
    }
}

fn setup_logger(level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                // "[ {} ][ {:5} ][ {:15} ] {}",
                // chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                "[ {:5} ][ {:15} ] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
