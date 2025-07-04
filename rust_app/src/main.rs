use rust_app;
use std::env;


#[tokio::main]
async fn main() {
    let mut _args:Vec<String> = env::args().collect();
    rust_app::run(&_args).await;
}
