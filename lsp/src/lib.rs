use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::*;
use std::error::Error;

pub struct Server {
    connection: Connection,
}

impl Server {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        eprintln!("start server");
        for msg in &self.connection.receiver {
            eprintln!("get msg: {:?}", msg);
        }

        Ok(())
    }
}
