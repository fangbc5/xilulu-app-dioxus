use dioxus::prelude::*;
use crate::router::Route;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdChevronLeft, LdSmartphone, LdRefreshCw};
use crate::i18n::use_language;
use rust_i18n::t;

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(ms: u64) {
    gloo_timers::future::sleep(std::time::Duration::from_millis(ms)).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

#[component]
pub fn OtpInput(length: usize, value: Signal<String>) -> Element {
    let mut focus = use_signal(|| false);
    
    rsx! {
        div { class: "relative w-full max-w-sm mx-auto",
            // The invisible real input
            input {
                class: "absolute inset-0 w-full h-full opacity-0 z-10 cursor-text",
                r#type: "text",
                inputmode: "numeric",
                maxlength: "{length}",
                value: "{value}",
                oninput: move |e| {
                    let val = e
                        .value()
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(length)
                        .collect::<String>();
                    value.set(val);
                },
                onfocus: move |_| focus.set(true),
                onblur: move |_| focus.set(false),
            }
            // The display boxes
            div { class: "flex justify-between gap-2 pointer-events-none",
                for i in 0..length {
                    div {
                        class: "w-11 h-14 sm:w-14 sm:h-16 flex items-center justify-center text-2xl font-bold rounded-xl border-2 transition-all",
                        class: if focus() && value().len() == i { "border-blue-500 ring-4 ring-blue-500/20 bg-white dark:bg-zinc-900" } else if value().len() > i { "border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 text-zinc-900 dark:text-white" } else { "border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/50 text-zinc-400" },
                        {
                            if let Some(c) = value().chars().nth(i) {
                                rsx! { "{c}" }
                            } else if focus() && value().len() == i {
                                rsx! {
                                    div { class: "w-0.5 h-6 bg-blue-500 animate-pulse" }
                                }
                            } else {
                                rsx! { "" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn OtpVerify(medium: String) -> Element {
    let code = use_signal(|| String::new());
    let code_len = 6;
    
    let mut countdown = use_signal(|| 60);

    let lang = use_language();
    let l_str = lang().as_str();

    use_future(move || async move {
        loop {
            if countdown() > 0 {
                sleep_ms(1000).await;
                countdown.with_mut(|c| *c -= 1);
            } else {
                sleep_ms(100).await;
            }
        }
    });

    let handle_verify = move |_| {
        let nav = use_navigator();
        // Mock verification: if code is 111111, go to AppHome (existing user)
        // Otherwise assume new user and go to ProfileSetup
        if code() == "111111" {
            nav.push(Route::AppHome {});
        } else {
            nav.push(Route::ProfileSetup {});
        }
    };

    let handle_resend = move |_| {
        if countdown() == 0 {
            countdown.set(60);
            // In real app, trigger resend API here
        }
    };

    rsx! {
        div { class: "h-[100dvh] w-full flex bg-zinc-50 dark:bg-[#0A0A0B] text-zinc-900 dark:text-zinc-100 overflow-hidden overscroll-none",
            // Desktop Left Panel (Hero area)
            div { class: "hidden lg:flex lg:w-1/2 relative bg-zinc-900 overflow-hidden items-center justify-center p-12 shrink-0",
                div { class: "absolute top-0 left-0 w-full h-full",
                    div { class: "absolute -top-[10%] -left-[10%] w-[60%] h-[60%] rounded-full bg-blue-600/20 blur-[100px]" }
                    div { class: "absolute bottom-[10%] -right-[10%] w-[50%] h-[50%] rounded-full bg-teal-500/20 blur-[100px]" }
                }

                div { class: "relative z-10 max-w-lg text-center",
                    div { class: "w-20 h-20 mx-auto bg-gradient-to-br from-blue-500 to-teal-400 rounded-3xl flex items-center justify-center mb-8 shadow-2xl shadow-blue-500/30",
                        Icon {
                            icon: LdSmartphone,
                            width: 32,
                            height: 32,
                            class: "text-white",
                        }
                    }
                    h1 { class: "text-4xl font-bold text-white mb-4", {t!("auth.verify_title", locale = l_str).to_string()} }
                    p { class: "text-lg text-zinc-400",
                        {t!("auth.verify_desc", locale = l_str).to_string()}
                    }
                }
            }

            // Right Panel
            div {
                class: "w-full lg:w-1/2 flex flex-col relative shrink-0",
                style: "padding-top: max(1rem, env(safe-area-inset-top)); padding-bottom: max(1rem, env(safe-area-inset-bottom));",

                // Top Bar
                div { class: "flex justify-start px-4 lg:px-6 pb-2 shrink-0 relative z-20",
                    div { class: "absolute inset-0 pointer-events-auto [user-drag:region] [-webkit-app-region:drag]" }
                    Link {
                        to: Route::Login {},
                        class: "w-10 h-10 rounded-full bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 flex items-center justify-center hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors relative z-30 [user-drag:none] [-webkit-app-region:no-drag]",
                        Icon {
                            icon: LdChevronLeft,
                            width: 20,
                            height: 20,
                            class: "text-zinc-700 dark:text-zinc-300",
                        }
                    }
                }

                // Internal scrolling wrapper
                div { class: "flex-1 overflow-y-auto overscroll-contain flex flex-col px-6 sm:px-12 lg:px-24 pb-8",
                    div { class: "my-auto w-full max-w-[420px] mx-auto z-10",
                        div { class: "mb-8 text-center",
                            h1 { class: "text-3xl font-bold tracking-tight mb-3", {t!("auth.enter_code", locale = l_str).to_string()} }
                            p { class: "text-zinc-500 dark:text-zinc-400 text-sm",
                                {t!("auth.sent_code_to", locale = l_str).to_string()} " "
                                span { class: "font-semibold text-zinc-900 dark:text-white",
                                    "{medium}"
                                }
                            }
                        }

                        div { class: "space-y-8",
                            // Use our new custom OTP Input component
                            OtpInput { length: code_len, value: code }

                            button {
                                class: "w-full py-4 bg-blue-600 hover:bg-blue-700 active:scale-[0.98] text-white rounded-2xl font-bold text-[15px] shadow-lg shadow-blue-500/25 transition-all text-center disabled:opacity-50",
                                disabled: code().len() < code_len,
                                onclick: handle_verify,
                                {t!("auth.verify_btn", locale = l_str).to_string()}
                            }

                            div { class: "flex justify-center",
                                button {
                                    class: "flex items-center gap-2 text-sm font-medium transition-colors",
                                    class: if countdown() > 0 { "text-zinc-400 cursor-not-allowed" } else { "text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300" },
                                    disabled: countdown() > 0,
                                    onclick: handle_resend,
                                    Icon {
                                        icon: LdRefreshCw,
                                        width: 14,
                                        height: 14,
                                    }
                                    if countdown() > 0 {
                                        {t!("auth.resend_in", locale=l_str, s=countdown()).to_string()}
                                    } else {
                                        {t!("auth.resend", locale=l_str).to_string()}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
