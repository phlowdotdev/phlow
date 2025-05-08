use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;
use phlow_sdk::prelude::*;
use tarpc::{client, context, tokio_serde::formats::Json};

use crate::{
    setup::{Config, StepInput},
    RPCClient,
};

pub async fn main_client(setup: ModuleSetup) -> anyhow::Result<()> {
    let config = Config::from(setup.with);

    let transports = {
        let mut transports = HashMap::new();

        for (name, server) in config.target_servers.iter() {
            let mut transport =
                tarpc::serde_transport::tcp::connect(server.get_address(), Json::default);

            transport.config_mut().max_frame_length(usize::MAX);

            transports.insert(name.clone(), Arc::new(transport.await?));
        }

        transports
    };

    let rx = module_channel!(setup);

    listen!(rx, move |package: ModulePackage| async {
        let input = match StepInput::try_from(package.input().unwrap_or(Value::Null)) {
            Ok(config) => config,
            Err(err) => {
                let response = ModuleResponse::from_error(err.to_string());
                sender_safe!(package.sender, response.into());
                return;
            }
        };

        let data = match transports.get(&input.server) {
            Some(transport) => {
                let client = RPCClient::new(client::Config::default(), transport.clone());

                client.call(context::current(), input.params).await;
            }
            None => {
                let response =
                    ModuleResponse::from_error(format!("Server {} not found", input.server));
                sender_safe!(package.sender, response.into());
                return;
            }
        };
    });

    Ok(())
}
