use dioxus::prelude::*;
use crate::i18n::use_language;
use rust_i18n::t;
use crate::router::Route;

#[component]
pub fn AppUser() -> Element {
    let lang = use_language();
    let l_str = lang().as_str();

    rsx! {
        div {
            class: "w-full min-h-screen flex flex-col pt-[env(safe-area-inset-top)] pb-[60px]",
            
            // Header / Profile Section placeholder
            div {
                class: "flex items-center p-6 bg-white dark:bg-[#191919] mb-2",
                div {
                    class: "w-16 h-16 rounded-lg bg-zinc-200 dark:bg-zinc-700 mr-4"
                }
                div {
                    class: "flex-1",
                    h2 { class: "text-xl font-bold text-zinc-900 dark:text-zinc-100", "微信用户" }
                    span { class: "text-sm text-zinc-500 mt-1", "微信号: user_123" }
                }
            }

            // Settings/Logout Section
            div {
                class: "mt-auto mb-4 px-4",
                button {
                    class: "w-full p-4 bg-white dark:bg-[#2C2C2C] text-[#FA5151] rounded-lg font-medium active:bg-zinc-50 dark:active:bg-zinc-800 transition-colors",
                    onclick: move |_| {
                        let nav = use_navigator();
                        let core_app = use_context::<std::sync::Arc<xilulu_im_sdk::service::app::CoreApp>>();
                        spawn(async move {
                            // Hit backend logout API and clean local storage and memory atomically
                            let _ = core_app.auth.logout().await;
                            nav.replace(Route::Login {});
                        });
                    },
                    {t!("auth.logout", locale=l_str).to_string()}
                }
            }
        }
    }
}
