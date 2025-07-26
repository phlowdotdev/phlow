mod config;
mod input;
mod jwt_handler;

use config::JwtConfig;
use input::JwtInput;
use jwt_handler::JwtHandler;
use phlow_sdk::prelude::*;

create_step!(jwt(setup));

/// JWT module entry point
pub async fn jwt(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse JWT configuration from 'with' parameters
    let config = match JwtConfig::try_from(setup.with) {
        Ok(config) => config,
        Err(e) => {
            log::debug!("JWT module configuration error: {}", e);
            return Err(e.into());
        }
    };

    log::debug!("JWT module started with config: {:?}", config);

    let jwt_handler = JwtHandler::new(config);

    for package in rx {
        let jwt_handler = jwt_handler.clone();

        let input = match JwtInput::try_from(package.input.clone()) {
            Ok(input) => input,
            Err(e) => {
                let response = ModuleResponse::from_error(format!("Invalid input: {}", e));
                sender_safe!(package.sender, response.into());
                continue;
            }
        };

        log::debug!("JWT module received input: {:?}", input);

        // Process based on action
        let result = match input {
            JwtInput::Create { data, expires_in } => {
                jwt_handler.create_token(data, expires_in).await
            }
            JwtInput::Verify { token } => jwt_handler.verify_token(token).await,
        };

        match result {
            Ok(response_value) => {
                log::debug!("JWT operation successful: {:?}", response_value);
                sender_safe!(package.sender, response_value.into());
            }
            Err(e) => {
                log::error!("JWT operation failed: {}", e);
                let response = ModuleResponse::from_error(format!("JWT operation failed: {}", e));
                sender_safe!(package.sender, response.into());
            }
        }
    }

    Ok(())
}
