use phlow_sdk::prelude::*;

create_step!(echo_bin(rx));

pub async fn echo_bin(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = echo(package.input().unwrap_or(Value::Null)).await;
        sender_safe!(package.sender, input.into());
    });

    Ok(())
}

pub async fn echo(input: Value) -> Value {
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use phlow_runtime::PhlowModule;
    use phlow_runtime::PhlowModuleSchema;
    use phlow_sdk::crossbeam;
    use phlow_sdk::tokio;
    use tokio::sync::oneshot;

    use phlow_engine::Context;
    use phlow_runtime::PhlowBuilder;
    use phlow_sdk::prelude::json;

    #[tokio::test]
    async fn runtime_echo_module() -> Result<(), Box<dyn std::error::Error>> {
        let pipeline = json!({
            "modules": [
                { "module": "echo",  }
            ],
            "steps": [
                { "use": "echo", "input": { "message": "hello" } }
            ]
        });

        let mut module = PhlowModule::new();
        module.set_schema(
            PhlowModuleSchema::new()
                .with_input(json!({ 
                    "type": "any",
                    "required": true,
                    "description": "The message to echo.",
                    "default": null
                 }))                                                    
                .with_output(json!({ 
                    "type": "any",
                    "required": true,
                    "description": "The echoed message.",
                    "default": null
                 }))
                .with_input_order(vec!["message"]),
        );
        module.set_handler(|request| async move {
            echo(request.input.unwrap_or(Value::Null)).await.into()
        });

        let mut builder = PhlowBuilder::new();
        builder.settings_mut().download = false;

        let mut runtime = builder
            .set_base_path(std::env::current_dir()?)
            .set_pipeline(pipeline)
            .set_context(Context::new())
            .set_module("echo", module)
            .build()
            .await?;

        let result = runtime.run().await?;
        runtime.shutdown().await?;

        assert_eq!(result, json!({ "message": "hello" }));
        Ok(())
    }

    #[tokio::test]
    async fn test_echo_with_string_input() {
        let (tx, rx) = crossbeam::channel::unbounded();
        let (result_tx, result_rx) = oneshot::channel();

        // Criar pacote com input string
        let package = ModulePackage {
            input: Some(Value::from("test message")),
            payload: None,
            sender: result_tx,
        };

        // Enviar pacote
        tx.send(package).unwrap();
        drop(tx);

        // Executar echo em background
        let echo_task = tokio::spawn(async move {
            echo_bin(rx).await.unwrap();
        });

        // Aguardar resultado
        let result = result_rx.await.unwrap();
        assert_eq!(result.data, Value::from("test message"));

        // Aguardar tarefa finalizar
        echo_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_echo_with_null_input() {
        let (tx, rx) = crossbeam::channel::unbounded();
        let (result_tx, result_rx) = oneshot::channel();

        // Criar pacote sem input
        let package = ModulePackage {
            input: None,
            payload: None,
            sender: result_tx,
        };

        // Enviar pacote
        tx.send(package).unwrap();
        drop(tx);

        // Executar echo em background
        let echo_task = tokio::spawn(async move {
            echo_bin(rx).await.unwrap();
        });

        // Aguardar resultado
        let result = result_rx.await.unwrap();
        assert_eq!(result.data, Value::Null);

        // Aguardar tarefa finalizar
        echo_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_echo_with_array_input() {
        let (tx, rx) = crossbeam::channel::unbounded();
        let (result_tx, result_rx) = oneshot::channel();

        let input = Value::from(vec!["item1", "item2", "item3"]);

        // Criar pacote com input array
        let package = ModulePackage {
            input: Some(input.clone()),
            payload: None,
            sender: result_tx,
        };

        // Enviar pacote
        tx.send(package).unwrap();
        drop(tx);

        // Executar echo em background
        let echo_task = tokio::spawn(async move {
            echo_bin(rx).await.unwrap();
        });

        // Aguardar resultado
        let result = result_rx.await.unwrap();
        assert_eq!(result.data, input);

        // Aguardar tarefa finalizar
        echo_task.await.unwrap();
    }
}
