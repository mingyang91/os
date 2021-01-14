use crate::println;

mod handler;
mod context;

pub fn init() {
    handler::init();
    println!("mod interrupt initialized");
}
