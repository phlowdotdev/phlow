pub struct Timer<'a> {
    name: &'a str,
    timer: std::time::Instant,
}

impl<'a> Timer<'a> {
    pub fn start(name: &'a str) -> Self {
        let timer = std::time::Instant::now();
        println!("Timer {} started", name);
        Self { name, timer }
    }

    pub fn stop(&self) {
        let elapsed = self.timer.elapsed();
        println!("Timer {} stopped, elapsed time: {:?}", self.name, elapsed);
    }
}
