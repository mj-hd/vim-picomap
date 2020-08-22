use smol::{future, io, Async};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use vim_picomap::server::*;

#[cfg(unix)]
fn main() -> io::Result<()> {
    let (signal_tx, signal_rx) = Async::<UnixStream>::pair()?;
    signal_hook::pipe::register(signal_hook::SIGTERM, signal_tx)?;

    let (done_tx, done_rx) = mpsc::channel();

    smol::run(async {
        future::race(
            async {
                let mut server = Server::default();

                match server.start(done_rx).await {
                    Err(result) => {
                        eprintln!("{:?}", result);

                        Err(result)
                    }
                    Ok(_) => Ok(()),
                }
            },
            async {
                signal_rx.readable().await?;

                eprintln!("received jobstop");

                done_tx.send(()).expect("failed to stop server");

                eprintln!("stopped!");

                Ok(())
            },
        )
        .await
        .expect("smol task exited unexpectedly");

        Ok(())
    })
}
