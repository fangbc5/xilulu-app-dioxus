use dioxus::prelude::*;
use crate::router::Route;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdCamera, LdUser};
use crate::i18n::use_language;
use rust_i18n::t;

#[component]
pub fn ProfileSetup() -> Element {
    let mut nickname = use_signal(|| String::new());
    let mut gender = use_signal(|| String::from("secret")); // male, female, secret

    let lang = use_language();
    let l_str = lang().as_str();

    let handle_finish = move |_| {
        let nav = use_navigator();
        nav.push(Route::AppHome {});
    };

    rsx! {
        div { class: "h-[100dvh] w-full flex bg-zinc-50 dark:bg-[#0A0A0B] text-zinc-900 dark:text-zinc-100 overflow-hidden overscroll-none",
            
            // Right pane (full on mobile)
            div { 
                class: "w-full max-w-[500px] mx-auto flex flex-col relative shrink-0",
                style: "padding-top: max(1rem, env(safe-area-inset-top)); padding-bottom: max(1rem, env(safe-area-inset-bottom));",

                div { class: "flex-1 overflow-y-auto overscroll-contain flex flex-col px-6 sm:px-12 pb-8",
                    div { class: "my-auto w-full z-10",
                        div { class: "text-center mb-10",
                            h1 { class: "text-3xl font-bold tracking-tight mb-3", {t!("auth.setup_title", locale=l_str).to_string()} }
                            p { class: "text-zinc-500 dark:text-zinc-400 text-sm", {t!("auth.setup_desc", locale=l_str).to_string()} }
                        }

                        div { class: "space-y-8",
                            // Avatar Picker
                            div { class: "flex justify-center",
                                button { class: "relative w-28 h-28 rounded-full bg-zinc-200 dark:bg-zinc-800 flex items-center justify-center border-4 border-white dark:border-[#0A0A0B] shadow-xl group overflow-hidden",
                                    Icon { icon: LdUser, width: 48, height: 48, class: "text-zinc-400 dark:text-zinc-600 group-hover:scale-110 transition-transform" }
                                    div { class: "absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity",
                                        Icon { icon: LdCamera, width: 24, height: 24, class: "text-white" }
                                    }
                                }
                            }

                            // Nickname Input
                            div { class: "space-y-2",
                                label { class: "text-sm font-semibold text-zinc-700 dark:text-zinc-300", {t!("auth.nickname", locale=l_str).to_string()} }
                                div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm",
                                    input {
                                        class: "flex-1 bg-transparent px-4 py-3.5 outline-none text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                        placeholder: "{t!(\"auth.nickname_holder\", locale=l_str)}",
                                        value: "{nickname}",
                                        oninput: move |e| nickname.set(e.value()),
                                    }
                                }
                            }

                            // Gender Selection
                            div { class: "space-y-3",
                                label { class: "text-sm font-semibold text-zinc-700 dark:text-zinc-300", {t!("auth.gender", locale=l_str).to_string()} }
                                div { class: "grid grid-cols-3 gap-3",
                                    button {
                                        class: "py-3 rounded-xl border text-sm font-medium transition-all text-center",
                                        class: if gender() == "male" { "bg-blue-50 dark:bg-blue-900/20 border-blue-500 text-blue-600 dark:text-blue-400" } else { "bg-white dark:bg-zinc-900/50 border-zinc-200 dark:border-zinc-800 text-zinc-600 dark:text-zinc-400" },
                                        onclick: move |_| gender.set("male".into()),
                                        {t!("auth.male", locale=l_str).to_string()}
                                    }
                                    button {
                                        class: "py-3 rounded-xl border text-sm font-medium transition-all text-center",
                                        class: if gender() == "female" { "bg-pink-50 dark:bg-pink-900/20 border-pink-500 text-pink-600 dark:text-pink-400" } else { "bg-white dark:bg-zinc-900/50 border-zinc-200 dark:border-zinc-800 text-zinc-600 dark:text-zinc-400" },
                                        onclick: move |_| gender.set("female".into()),
                                        {t!("auth.female", locale=l_str).to_string()}
                                    }
                                    button {
                                        class: "py-3 rounded-xl border text-sm font-medium transition-all text-center",
                                        class: if gender() == "secret" { "bg-zinc-100 dark:bg-zinc-800 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-white" } else { "bg-white dark:bg-zinc-900/50 border-zinc-200 dark:border-zinc-800 text-zinc-600 dark:text-zinc-400" },
                                        onclick: move |_| gender.set("secret".into()),
                                        {t!("auth.secret", locale=l_str).to_string()}
                                    }
                                }
                            }

                            button {
                                class: "w-full mt-6 py-4 bg-zinc-900 dark:bg-white hover:bg-zinc-800 dark:hover:bg-zinc-200 active:scale-[0.98] text-white dark:text-zinc-900 rounded-2xl font-bold text-[15px] shadow-lg shadow-zinc-500/25 transition-all text-center disabled:opacity-50",
                                disabled: nickname().is_empty(),
                                onclick: handle_finish,
                                {t!("auth.complete", locale=l_str).to_string()}
                            }
                        }
                    }
                }
            }
        }
    }
}
