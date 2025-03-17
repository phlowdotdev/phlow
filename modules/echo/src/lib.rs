use std::sync::mpsc::channel;

use sdk::{modules::ModulePackage, prelude::*};

plugin_async!(echo);

pub async fn echo(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::<ModulePackage>();

    setup.setup_sender.send(Some(tx)).unwrap();

    println!("echo start_server!");

    for package in rx {
        println!("echo received package!");
        package
            .sender
            .send(Value::from("Hello, World!".to_string()))
            .unwrap();
    }

    Ok(())
}
