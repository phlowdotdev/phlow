use sdk::prelude::*;

plugin!(echo);

pub fn echo(_setup: ModuleSetup) {
    println!("echo start_server");
}
