use flutter_rust_bridge::frb;
use xilulu_core;

#[frb(sync)]
pub fn init_app() {
    // Initialize things here
    flutter_rust_bridge::setup_default_user_utils();
}

pub async fn ping_core() -> String {
    "Pong from Xilulu Core!".to_string()
}
