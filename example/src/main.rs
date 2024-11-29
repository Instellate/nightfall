use nightfall_macros::command_controller;
use std::error::Error;

struct Test;

#[command_controller]
impl Test {
    async fn testy(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

fn main() {
}
