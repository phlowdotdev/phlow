use valu3::prelude::*;

#[no_mangle]
pub extern "C" fn process_data(data: *const Value) {
    unsafe {
        if data.is_null() {
            return;
        }

        let data_ref = &*data;

        if let Value::Object(map) = data_ref {
            for (k, v) in map.iter() {
                println!("{}: {}", k, v);
            }
        }
    }
}
