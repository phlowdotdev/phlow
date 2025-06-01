use futures::channel::oneshot::{self, Sender};
use phlow_sdk::{crossbeam, prelude::*};
use std::collections::HashMap;
use tarpc::{client, context, tokio_serde::formats::Json};

use crate::{
    setup::{Config, StepInput},
    RPCClient,
};

struct InnerTransport {
    tx: Sender<ModuleResponse>,
    data: Value,
}

pub async fn main(rx: channel::Receiver<ModulePackage>, config: Config) {
    let mut transports = HashMap::new();
    let mut _spawns_target = Vec::new();

    for (name, server) in config.target_servers.iter() {
        let mut transport =
            tarpc::serde_transport::tcp::connect(server.get_address(), Json::default);

        transport.config_mut().max_frame_length(usize::MAX);

        let client = RPCClient::new(client::Config::default(), transport.await.unwrap()).spawn();

        let (transport_tx, transport_rx) = crossbeam::channel::unbounded::<InnerTransport>();

        _spawns_target.push(tokio::spawn(async move {
            for _ in 1..config.parallel_executions {
                for input in transport_rx.clone() {
                    let response = match client.call(context::current(), input.data).await {
                        Ok(value) => ModuleResponse::from_success(value),
                        Err(err) => ModuleResponse::from_error(err.to_string()),
                    };

                    input.tx.send(response).unwrap();
                }
            }
        }));

        transports.insert(name.clone(), transport_tx);
    }

    for _ in 1..config.parallel_executions {
        for package in rx.clone() {
            let input = match StepInput::try_from(package.input().unwrap_or(Value::Null)) {
                Ok(config) => config,
                Err(err) => {
                    let response = ModuleResponse::from_error(err.to_string());
                    sender_safe!(package.sender, response.into());
                    return;
                }
            };

            match transports.get(&input.server) {
                Some(transport) => {
                    let (tx, rx) = oneshot::channel::<ModuleResponse>();

                    let inner = InnerTransport {
                        tx,
                        data: input.params,
                    };

                    if let Err(err) = transport.send(inner) {
                        let response = ModuleResponse::from_error(err.to_string());
                        sender_safe!(package.sender, response.into());
                        return;
                    }

                    let response = match rx.await {
                        Ok(value) => value,
                        Err(err) => ModuleResponse::from_error(err.to_string()),
                    };

                    sender_safe!(package.sender, response.into());
                }
                None => {
                    let response = ModuleResponse::from_error("Server not found".to_string());
                    sender_safe!(package.sender, response.into());
                }
            }
        }
    }
}
