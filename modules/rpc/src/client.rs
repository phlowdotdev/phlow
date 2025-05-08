// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use phlow_sdk::structs::ModuleSetup;
use std::{net::SocketAddr, time::Duration};
use tarpc::{client, context, tokio_serde::formats::Json};
use tokio::time::sleep;

use crate::{setup::Config, WorldClient};

pub async fn main(config: Config) -> anyhow::Result<()> {
    let mut transport = tarpc::serde_transport::tcp::connect(config.server_addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    let hello = hello
        .hello(context::current(), format!("{}1", flags.name))
        .await;

    Ok(())
}
