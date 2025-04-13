mod args;
mod resolve;
use std::env;

use args::Args;
use phlow_sdk::prelude::*;
use resolve::resolve;

create_main!(cli(setup));

pub async fn cli(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sender_safe!(setup.setup_sender, None);

    let _ = phlow_sdk::tracing::dispatcher::with_default(&setup.dispatch.clone(), || async move {
        let span = tracing::span!(
            Level::INFO,
            "cli_command",
            otel.name = setup.app_data.name.as_deref().unwrap_or("phlow cli"),
            "process.executable.name" = env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            "process.exit.code" = field::Empty,
            "error.type" = field::Empty,
            "process.pid" = std::process::id(),
            "process.command_args" = env::args().collect::<Vec<String>>().join(" "),
            "process.executable.path" = env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        );

        span_enter!(span);

        let with_args = setup.with.get("args").cloned().unwrap_or(Value::Null);
        let additional_args = *setup
            .with
            .get("additional_args")
            .cloned()
            .unwrap_or(Value::Boolean(false))
            .as_bool()
            .unwrap_or(&false);

        let args = Args::new(with_args, setup.app_data.clone(), additional_args);

        if args.is_error() {
            span.record("error.type", "invalid_input");
            span.record("process.exit.code", 1);
            args.print_error_with_help();
            return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
        }

        if args.is_help() {
            span.record("process.exit.code", 0);
            args.print_help(None);
            return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
        }

        let context = resolve::RequestContext {
            args: args.clone(),
            span: span.clone(),
            dispatch: setup.dispatch.clone(),
            id: setup.id,
            sender: setup.main_sender.unwrap(),
        };

        let response = resolve(context).await;

        if response.is_err() {
            span.record("error.type", "resolve_error");
            span.record("process.exit.code", 1);
            eprintln!("Error: {:?}", response.err());
        } else {
            span.record("process.exit.code", 0);
            let value = response.unwrap();
            println!("{}", value);
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await;

    Ok(())
}
