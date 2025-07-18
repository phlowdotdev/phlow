use phlow_sdk::prelude::*;

create_step!(echo(rx));

pub async fn echo(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    

    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        sender_safe!(package.sender, input.into());
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use phlow_sdk::crossbeam;
    use phlow_sdk::tokio;
    use tokio::sync::oneshot;

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
            echo(rx).await.unwrap();
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
            echo(rx).await.unwrap();
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
            echo(rx).await.unwrap();
        });

        // Aguardar resultado
        let result = result_rx.await.unwrap();
        assert_eq!(result.data, input);

        // Aguardar tarefa finalizar
        echo_task.await.unwrap();
    }
}
