use sdk::prelude::*;

#[no_mangle]
pub extern "C" fn process_data(data: *const Value, callback: CallbackFn) {
    unsafe {
        if data.is_null() {
            return;
        }

        let data_ref = &*data;

        if let Value::Object(map) = data_ref {
            for (k, v) in map.iter() {
                println!("{}: {}", k, v);
            }

            let callback_value = Value::from("Plugin!");
            let boxed_callback_value = Box::new(callback_value);

            let result_ptr = callback(Box::into_raw(boxed_callback_value));

            if !result_ptr.is_null() {
                let result_ref = &*result_ptr;
                println!("Plugin: {:?}", result_ref);
            }
        }
    }
}
