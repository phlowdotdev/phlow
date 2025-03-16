use sdk::prelude::*;

plugin!(echo);

pub fn echo(_id: ModuleId, _sender: MainRuntimeSender, _setup: Value) {
    println!("echo start_server");
}
