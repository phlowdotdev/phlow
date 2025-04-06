use phlow_sdk::prelude::*;

create_step!(sleep);

struct Sleep {
    time: u64,
}

impl From<&Value> for Sleep {
    fn from(value: &Value) -> Self {
        let time = if let Some(value) = value.get("milliseconds") {
            value.to_u64().unwrap_or(0)
        } else if let Some(value) = value.get("seconds") {
            value.to_u64().unwrap_or(0) * 1000
        } else if let Some(value) = value.get("minutes") {
            value.to_u64().unwrap_or(0) * 1000 * 60
        } else if let Some(value) = value.get("hours") {
            value.to_u64().unwrap_or(0) * 1000 * 60
        } else {
            0
        };

        Self { time }
    }
}

pub async fn sleep(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    match setup.setup_sender.send(Some(tx)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e).into());
        }
    };

    for package in rx {
        let sleep = match package.context.input {
            Some(value) => Sleep::from(&value),
            _ => Sleep { time: 0 },
        };

        if sleep.time > 0 {
            debug!("Sleeping for {} milliseconds", sleep.time);
            std::thread::sleep(std::time::Duration::from_millis(sleep.time));
        } else {
            debug!("No sleep time provided, skipping sleep");
        }

        sender_safe!(package.sender, Value::Null);
    }

    Ok(())
}
