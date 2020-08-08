use signal_hook::iterator::Signals;
use smol::{future, io};
use std::sync::mpsc;
use vim_picomap::server::Server;

#[cfg(unix)]
fn main() -> io::Result<()> {
    let signals = Signals::new(&[
        signal_hook::SIGINT,
        signal_hook::SIGTERM,
        signal_hook::SIGQUIT,
    ])?;

    let (exit_tx, exit_rx) = mpsc::channel();

    smol::run(async {
        future::race(
            async {
                let mut server = Server::default();

                match server.start(&exit_rx) {
                    Err(result) => {
                        eprintln!("{:?}", result);

                        Err(result)
                    }
                    Ok(_) => Ok(()),
                }
            },
            async {
                signals.forever().next();

                eprintln!("received jobstop");

                exit_tx.send(()).expect("failed to stop server");

                eprintln!("stopped!");

                Ok(())
            },
        )
        .await
        .expect("smol task exited unexpectedly");

        Ok(())
    })
}
