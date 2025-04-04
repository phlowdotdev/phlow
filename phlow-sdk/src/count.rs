pub struct Count {
    count: u64,
}

impl Count {
    pub fn new() -> Self {
        Count { count: 0 }
    }

    pub fn add(&mut self) {
        self.count += 1;
    }

    pub fn sub(&mut self) {
        self.count -= 1;
    }

    pub fn get(&self) -> u64 {
        self.count
    }
}

#[macro_export]
macro_rules! counter_listener {
    ($counter:expr, $time:expr) => {
        let counter_clone = counter.clone();
        tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                let count = counter_clone.lock().unwrap().get();
                println!("Current count: {}", count);
            }
        });
    };
}
