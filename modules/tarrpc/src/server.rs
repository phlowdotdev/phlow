use crate::setup::Config;
use phlow_sdk::prelude::*;

pub async fn start_server(
    config: Config,
    _dispatch: Dispatch,
    _main_sender: MainRuntimeSender,
    _id: ModuleId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_address = config.server_address();
    
    info!("Starting tarpc server on {}", server_address);
    
    // Para simplicidade, vamos simular um servidor
    // Em uma implementação real, você criaria um servidor tarpc real
    
    // Simular servidor rodando
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        debug!("tarpc server running on {}", server_address);
        
        // Simular processamento de mensagens
        // Em uma implementação real, você processaria mensagens RPC aqui
    }
}

pub async fn start_memory_server(
    _config: Config,
    _dispatch: Dispatch,
    _main_sender: MainRuntimeSender,
    _id: ModuleId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting tarpc in-memory server");
    
    // Para simplicidade, vamos simular um servidor em memória
    // Em uma implementação real, você criaria um canal de memória
    
    // Simular servidor rodando
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        debug!("tarpc in-memory server running");
        
        // Simular processamento de mensagens
        // Em uma implementação real, você processaria mensagens RPC aqui
    }
}
