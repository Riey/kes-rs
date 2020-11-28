use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (connection, io_threads) = Connection::stdio();

    kes_lsp::Server::new(connection).run()?;

    io_threads.join()?;

    Ok(())
}
