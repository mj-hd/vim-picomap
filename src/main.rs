use vim_micromap::server::Server;

fn main() {
    let mut server = Server::default();

    if let Err(result) = server.start() {
        eprintln!("{:?}", result);
    }
}
