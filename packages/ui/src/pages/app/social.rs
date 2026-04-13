use dioxus::prelude::*;

#[component]
pub fn AppSocial() -> Element {
    rsx! {
        div {
            class: "w-full h-full flex items-center justify-center pt-[env(safe-area-inset-top)]",
            div {
                class: "text-zinc-500",
                "发现页面占位 - 待开发"
            }
        }
    }
}
