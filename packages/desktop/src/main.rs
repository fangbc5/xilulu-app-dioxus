use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Xilulu Auth")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(400.0, 700.0))
                    .with_resizable(false)
                    .with_maximizable(false),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        ui::router::AppRoot {}
    }
}
