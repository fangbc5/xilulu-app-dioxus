use dioxus::prelude::*;

#[component]
fn TestApp() -> Element {
    let mut num = use_signal(|| 0);
    rsx! {
        div {
            button {
                onclick: move |_| {
                    let w = dioxus::desktop::window();
                    w.new_window(dioxus::prelude::VirtualDom::new(TestApp), dioxus::desktop::Config::new());
                    w.close();
                },
                "Spawn"
            }
        }
    }
}
