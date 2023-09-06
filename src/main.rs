mod i3;

use crate::i3::I3Manager;
use anyhow::{Context, Error, Result};
use log::{error, info};
use xdg::BaseDirectories;

fn main() -> Result<()> {
    info!("Starting i3helper version {}", env!("CARGO_PKG_VERSION"));

    let sock_path = BaseDirectories::new()
        .context("failed to read XDG directories")?
        .place_runtime_file("i3helperd.sock")
        .context("failed to create socket file")?;

    // if let Err(err) = async {
    //     let listener = UnixListener::bind(&sock_path)?;
    //     info!(
    //         "Listening for unix domain socket connections at `{}`",
    //         listener
    //             .local_addr()?
    //             .as_pathname()
    //             .expect("Can't convert local socket to path name")
    //             .display()
    //     );
    //
    //     let mut i3_manager = I3Manager::new(listener).await?;
    //     i3_manager.run().await?;
    //     Ok::<(), Error>(())
    // }
    // .await
    // {
    //     error!("Something went wrong: {}", err);
    // }

    std::fs::remove_file(sock_path)?;

    Ok(())
}
