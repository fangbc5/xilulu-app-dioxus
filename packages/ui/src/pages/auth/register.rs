use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdChevronLeft, LdLock, LdSmartphone, LdUser};
use dioxus_free_icons::Icon;
use crate::i18n::use_language;
use rust_i18n::t;

#[component]
pub fn Register() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut captcha = use_signal(|| String::new());

    let lang = use_language();
    let l_str = lang().as_str();

    let handle_register = move |_| {
        // Rediect to ProfileSetup on success
        let navigator = use_navigator();
        navigator.push(Route::ProfileSetup {});
    };

    rsx! {
        div { class: "h-[100dvh] w-full flex bg-zinc-50 dark:bg-[#0A0A0B] text-zinc-900 dark:text-zinc-100 overflow-hidden overscroll-none",

            // Desktop Left Panel (Hero area)
            div { class: "hidden lg:flex lg:w-1/2 relative bg-zinc-900 overflow-hidden items-center justify-center p-12",
                // Decorative animated gradients inside the hero
                div { class: "absolute top-0 left-0 w-full h-full",
                    div { class: "absolute top-[10%] right-[10%] w-[50%] h-[50%] rounded-full bg-purple-600/20 blur-[100px]" }
                    div { class: "absolute bottom-[20%] left-[10%] w-[40%] h-[40%] rounded-full bg-blue-500/20 blur-[100px]" }
                }

                div { class: "relative z-10 max-w-lg text-center",
                    h2 { class: "text-4xl font-bold text-white mb-6", "Join the community" }
                    p { class: "text-lg text-zinc-400 mb-8",
                        "Create your account and unlock the full potential of seamless collaboration and real-time communication."
                    }

                    div { class: "grid grid-cols-2 gap-4 text-left",
                        div { class: "bg-white/5 border border-white/10 rounded-2xl p-5 backdrop-blur-sm",
                            div { class: "w-8 h-8 rounded-full bg-purple-500/20 flex items-center justify-center mb-3",
                                Icon {
                                    icon: LdSmartphone,
                                    width: 16,
                                    height: 16,
                                    class: "text-purple-400",
                                }
                            }
                            h3 { class: "text-white font-medium mb-1", "Cross Platform" }
                            p { class: "text-xs text-zinc-400", "Access from anywhere" }
                        }
                        div { class: "bg-white/5 border border-white/10 rounded-2xl p-5 backdrop-blur-sm",
                            div { class: "w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center mb-3",
                                Icon {
                                    icon: LdLock,
                                    width: 16,
                                    height: 16,
                                    class: "text-blue-400",
                                }
                            }
                            h3 { class: "text-white font-medium mb-1", "Secure" }
                            p { class: "text-xs text-zinc-400", "End-to-end encryption" }
                        }
                    }
                }
            }

            // Right Panel (Mobile Full, Desktop Half)
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

                // Scrollable Form Container
                div { class: "flex-1 overflow-y-auto overscroll-contain flex flex-col px-6 sm:px-12 lg:px-24 pb-8",
                    div { class: "my-auto w-full max-w-[420px] mx-auto z-10",

                        // Header
                        div { class: "mb-8",
                            h1 { class: "text-3xl font-bold tracking-tight mb-2",
                                {t!("auth.register_title", locale = l_str).to_string()}
                            }
                            p { class: "text-zinc-500 dark:text-zinc-400",
                                {t!("auth.register_desc", locale = l_str).to_string()}
                            }
                        }

                        // Input Section
                        div { class: "space-y-4 mb-8",
                            div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-purple-500/20 focus-within:border-purple-500 shadow-sm",
                                div { class: "pl-4 text-zinc-400",
                                    Icon { icon: LdUser, width: 18, height: 18 }
                                }
                                input {
                                    r#type: "text",
                                    class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                    placeholder: "{t!(\"auth.enter_only_username\", locale = l_str)}",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value()),
                                }
                            }
                            div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-purple-500/20 focus-within:border-purple-500 shadow-sm",
                                div { class: "pl-4 text-zinc-400",
                                    Icon { icon: LdLock, width: 18, height: 18 }
                                }
                                input {
                                    r#type: "password",
                                    class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                    placeholder: "{t!(\"auth.enter_password\", locale = l_str)}",
                                    value: "{password}",
                                    oninput: move |e| password.set(e.value()),
                                }
                            }
                            div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-purple-500/20 focus-within:border-purple-500 shadow-sm",
                                div { class: "pl-4 text-zinc-400",
                                    Icon { icon: LdLock, width: 18, height: 18 }
                                }
                                input {
                                    r#type: "password",
                                    class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                    placeholder: "{t!(\"auth.enter_confirm_password\", locale = l_str)}",
                                    value: "{confirm_password}",
                                    oninput: move |e| confirm_password.set(e.value()),
                                }
                            }
                            div { class: "flex gap-2",
                                div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-purple-500/20 focus-within:border-purple-500 shadow-sm flex-1",
                                    input {
                                        r#type: "text",
                                        class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400 text-sm",
                                        placeholder: "{t!(\"auth.enter_captcha\", locale = l_str)}",
                                        value: "{captcha}",
                                        oninput: move |e| captcha.set(e.value()),
                                    }
                                }
                                button { class: "w-28 bg-zinc-100 dark:bg-zinc-800 rounded-2xl flex items-center justify-center text-xl font-bold tracking-widest text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors shrink-0",
                                    style: "font-family: monospace; background-image: url('data:image/svg+xml;utf8,<svg width=\"100%\" height=\"100%\" xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M0 0l20 20M30 10l-15 40\" stroke=\"#888\" stroke-width=\"1\" fill=\"none\"/></svg>');",
                                    "3F2A"
                                }
                            }
                        }

                        // Action
                        button {
                            class: "w-full py-3.5 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 hover:bg-zinc-800 dark:hover:bg-zinc-100 active:scale-[0.98] rounded-2xl font-semibold shadow-lg shadow-zinc-500/10 transition-all text-[15px]",
                            onclick: handle_register,
                            {t!("auth.sign_up", locale = l_str).to_string()}
                        }
                                        // End my-auto wrap
                    }
                }
            }
        }
    }
}
