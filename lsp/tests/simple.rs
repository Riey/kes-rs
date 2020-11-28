use kes_lsp::Server;
use lsp_server::{Connection, Message, Request, RequestId};
use lsp_types::*;
use serde_json::Value;

struct Client {
    connection: Connection,
    id: i32,
}

impl Client {
    pub fn new(connection: Connection) -> Self {
        Self { connection, id: 0 }
    }

    fn next_id(&mut self) -> RequestId {
        let id = self.id;
        self.id += 1;
        id.into()
    }

    pub fn send<R>(&mut self, params: R::Params) -> Value
    where
        R: request::Request,
    {
        let id = self.next_id();

        let req = Request::new(id.clone(), R::METHOD.into(), params);

        self.connection.sender.send(Message::Request(req)).unwrap();

        loop {
            let msg = self
                .connection
                .receiver
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap();

            match msg {
                Message::Request(..) => {
                    // TODO:
                }
                Message::Notification(_) => {}
                Message::Response(res) => {
                    assert_eq!(res.id, id);

                    return res.result.unwrap();
                }
            }
        }
    }
}

#[test]
fn run_test() {
    let (client, server) = Connection::memory();
    std::thread::Builder::new()
        .name("server".into())
        .spawn(move || {
            Server::new(server).run().unwrap();
        })
        .unwrap();

    let mut client = Client::new(client);

    client.send::<request::GotoDefinition>(GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            position: Position::new(0, 0),
            text_document: TextDocumentIdentifier::new(Url::parse("file://foo.kes").unwrap()),
        },
        work_done_progress_params: WorkDoneProgressParams {
            work_done_token: None,
        },
        partial_result_params: PartialResultParams {
            partial_result_token: None,
        },
    });
}
