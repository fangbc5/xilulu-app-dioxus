use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdChevronDown, LdEye, LdEyeOff, LdGlobe, LdLock, LdMail, LdSmartphone,
};
use dioxus_free_icons::Icon;
use crate::i18n::{Language, use_language};
use rust_i18n::t;

#[component]
pub fn Login() -> Element {
    let mut active_tab = use_signal(|| "phone");
    let mut phone = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut captcha = use_signal(|| String::new());
    let mut show_password = use_signal(|| false);
    let mut agreed = use_signal(|| true);

    let mut lang = use_language();
    let l_str = lang().as_str();

    #[cfg(feature = "desktop")]
    {
        use_effect(move || {
            let window = dioxus::desktop::window();
            window.set_inner_size(dioxus::desktop::LogicalSize::new(400.0, 700.0));
            window.set_resizable(false);
            window.set_maximizable(false);
        });
    }

    let is_phone = active_tab() == "phone";
    let is_email = active_tab() == "email";
    let is_password = active_tab() == "password";

    let handle_next = move |_| {
        if !agreed() {
            // handle error
            return;
        }
        
        let navigator = use_navigator();
        
        if active_tab() == "password" {
            // Success password login logic
            navigator.push(Route::AppHome {});
        } else {
            let medium = if active_tab() == "phone" { phone() } else { email() };
            let display_medium = if medium.is_empty() { "your account".to_string() } else { medium };
            navigator.push(Route::OtpVerify { medium: display_medium });
        }
    };

    rsx! {
        div { class: "h-[100dvh] w-full flex bg-zinc-50 dark:bg-[#0A0A0B] text-zinc-900 dark:text-zinc-100 overflow-hidden overscroll-none",

            // Desktop Left Panel (Hero area)
            div { class: "hidden lg:flex lg:w-1/2 relative bg-zinc-900 overflow-hidden items-center justify-center p-12",
                // Decorative animated gradients inside the hero
                div { class: "absolute top-0 left-0 w-full h-full",
                    div { class: "absolute -top-[10%] -left-[10%] w-[60%] h-[60%] rounded-full bg-blue-600/20 blur-[100px]" }
                    div { class: "absolute bottom-[10%] -right-[10%] w-[50%] h-[50%] rounded-full bg-teal-500/20 blur-[100px]" }
                }

                div { class: "relative z-10 max-w-lg",
                    div { class: "w-16 h-16 bg-gradient-to-br from-blue-500 to-teal-400 rounded-2xl flex items-center justify-center mb-8 shadow-2xl shadow-blue-500/30",
                        span { class: "text-3xl font-bold text-white", "X" }
                    }
                    h1 { class: "text-5xl font-bold text-white mb-6 leading-tight", "Welcome to the future of collaboration." }
                    p { class: "text-xl text-zinc-400 leading-relaxed", "Xilulu brings all your team's conversations, meetings, and documents into one unified, seamless experience." }

                    // Decorative mock UI below
                    div { class: "mt-16 bg-white/5 border border-white/10 rounded-2xl p-6 backdrop-blur-sm",
                        div { class: "flex items-center gap-4 mb-4",
                            div { class: "w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center",
                                Icon { icon: LdGlobe, width: 20, height: 20, class: "text-blue-400" }
                            }
                            div {
                                div { class: "h-4 w-32 bg-white/20 rounded mb-2" }
                                div { class: "h-3 w-24 bg-white/10 rounded" }
                            }
                        }
                    }
                }
            }

            // Right Panel (Mobile Full, Desktop Half)
            div {
                class: "w-full lg:w-1/2 flex flex-col relative shrink-0",
                style: "padding-top: max(1rem, env(safe-area-inset-top)); padding-bottom: max(1rem, env(safe-area-inset-bottom));",

                // Top Bar with Drag region (desktop) and safe area (mobile)
                div { class: "flex justify-end px-6 pb-2 shrink-0 relative z-20",
                    // Desktop window drag handle helper
                    div { class: "absolute inset-0 pointer-events-auto [user-drag:region] [-webkit-app-region:drag]" }
                    // Lang switch
                    button { 
                        class: "flex items-center gap-1.5 text-sm text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-300 transition-colors relative z-30 [user-drag:none] [-webkit-app-region:no-drag] bg-white/50 dark:bg-black/50 px-3 py-1.5 rounded-full border border-zinc-200 dark:border-zinc-800 shadow-sm",
                        onclick: move |_| {
                            let new_lang = if lang() == Language::Zh { Language::En } else { Language::Zh };
                            *lang.write() = new_lang;
                        },
                        Icon { icon: LdGlobe, width: 16, height: 16 }
                        span { {if lang() == Language::Zh { "English" } else { "中文" }} }
                    }
                }

                // Scrollable Form Container within the right pane
                div { class: "flex-1 overflow-y-auto overscroll-contain flex flex-col px-6 sm:px-12 lg:px-24 pb-8",
                    div { class: "my-auto w-full max-w-[420px] mx-auto z-10",

                    // Logo & Header (Center on mobile, Left on desktop)
                    div { class: "mb-8 text-center lg:text-left",
                        div { class: "w-14 h-14 mx-auto lg:mx-0 bg-gradient-to-br from-blue-500 to-teal-400 rounded-2xl shadow-xl shadow-blue-500/20 mb-5 flex items-center justify-center lg:hidden",
                            span { class: "text-2xl font-bold text-white", "X" }
                        }
                        h1 { class: "text-3xl font-bold tracking-tight mb-2", {t!("auth.login_title", locale = l_str).to_string()} }
                    p { class: "text-zinc-500 dark:text-zinc-400",
                        {t!("auth.login_desc", locale = l_str).to_string()}
                    }
                }

                // Tabs
                div { class: "flex w-full mb-8 border-b border-zinc-200 dark:border-zinc-800",
                    button {
                        class: "flex-1 pb-3 text-sm font-medium transition-all relative",
                        class: if is_phone { "text-blue-600 dark:text-blue-400" } else { "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-300" },
                        onclick: move |_| active_tab.set("phone"),
                        {t!("auth.phone", locale = l_str).to_string()}
                        if is_phone {
                            div { class: "absolute bottom-0 left-0 w-full h-0.5 bg-blue-600 dark:bg-blue-400 rounded-t-full shadow-[0_-2px_10px_rgba(37,99,235,0.5)]" }
                        }
                    }
                    button {
                        class: "flex-1 pb-3 text-sm font-medium transition-all relative",
                        class: if is_email { "text-blue-600 dark:text-blue-400" } else { "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-300" },
                        onclick: move |_| active_tab.set("email"),
                        {t!("auth.email", locale = l_str).to_string()}
                        if is_email {
                            div { class: "absolute bottom-0 left-0 w-full h-0.5 bg-blue-600 dark:bg-blue-400 rounded-t-full shadow-[0_-2px_10px_rgba(37,99,235,0.5)]" }
                        }
                    }
                    button {
                        class: "flex-1 pb-3 text-sm font-medium transition-all relative",
                        class: if is_password { "text-blue-600 dark:text-blue-400" } else { "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-300" },
                        onclick: move |_| active_tab.set("password"),
                        {t!("auth.password", locale = l_str).to_string()}
                        if is_password {
                            div { class: "absolute bottom-0 left-0 w-full h-0.5 bg-blue-600 dark:bg-blue-400 rounded-t-full shadow-[0_-2px_10px_rgba(37,99,235,0.5)]" }
                        }
                    }
                }

                // Input Section
                div { class: "space-y-4 mb-8",
                    if is_phone {
                        div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm",
                            button { class: "flex items-center gap-1.5 pl-4 pr-3 py-3 text-zinc-700 dark:text-zinc-300 font-medium",
                                span { "+86" }
                                Icon {
                                    icon: LdChevronDown,
                                    width: 14,
                                    height: 14,
                                    class: "text-zinc-400",
                                }
                            }
                            div { class: "w-px h-5 bg-zinc-200 dark:bg-zinc-800" }
                            input {
                                class: "flex-1 bg-transparent px-3 py-3.5 outline-none text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                placeholder: "{t!(\"auth.enter_phone\", locale = l_str)}",
                                value: "{phone}",
                                oninput: move |e| phone.set(e.value()),
                            }
                        }
                    } else if is_email {
                        div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm",
                            div { class: "pl-4 text-zinc-400",
                                Icon { icon: LdMail, width: 18, height: 18 }
                            }
                            input {
                                r#type: "email",
                                class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                placeholder: "{t!(\"auth.enter_email\", locale = l_str)}",
                                value: "{email}",
                                oninput: move |e| email.set(e.value()),
                            }
                        }
                    } else {
                        div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm",
                            div { class: "pl-4 text-zinc-400",
                                Icon {
                                    icon: LdSmartphone,
                                    width: 18,
                                    height: 18,
                                }
                            }
                            input {
                                r#type: "text",
                                class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                placeholder: "{t!(\"auth.enter_username\", locale = l_str)}",
                                value: "{username}",
                                oninput: move |e| username.set(e.value()),
                            }
                        }
                        div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm mt-3",
                            div { class: "pl-4 text-zinc-400",
                                Icon { icon: LdLock, width: 18, height: 18 }
                            }
                            input {
                                r#type: if show_password() { "text" } else { "password" },
                                class: "flex-1 bg-transparent px-4 py-3 outline-none text-zinc-900 dark:text-white placeholder:text-zinc-400",
                                placeholder: "{t!(\"auth.enter_password\", locale = l_str)}",
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                            }
                            button {
                                class: "pr-4 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200 transition-colors",
                                onclick: move |_| show_password.set(!show_password()),
                                if show_password() {
                                    Icon {
                                        icon: LdEyeOff,
                                        width: 18,
                                        height: 18,
                                    }
                                } else {
                                    Icon { icon: LdEye, width: 18, height: 18 }
                                }
                            }
                        }
                        div { class: "flex gap-2 mt-3",
                            div { class: "relative flex items-center bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-1 transition-all focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 shadow-sm flex-1",
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
                }

                // Terms
                label { class: "flex items-center gap-3 mb-8 cursor-pointer group",
                    div {
                        class: "w-5 h-5 rounded flex items-center justify-center transition-colors border",
                        class: if agreed() { "bg-blue-600 border-blue-600" } else { "border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900" },
                        onclick: move |_| agreed.set(!agreed()),
                        if agreed() {
                            svg {
                                class: "w-3 h-3 text-white",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                stroke_width: "3",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M5 13l4 4L19 7",
                                }
                            }
                        }
                    }
                    span { class: "text-sm text-zinc-500",
                        {t!("auth.agree_part1", locale = l_str).to_string()}
                        Link { to: Route::Terms {}, class: "text-blue-600 dark:text-blue-400 hover:underline",
                            {t!("auth.terms", locale = l_str).to_string()}
                        }
                        {t!("auth.agree_part2", locale = l_str).to_string()}
                        Link { to: Route::Privacy {}, class: "text-blue-600 dark:text-blue-400 hover:underline",
                            {t!("auth.privacy", locale = l_str).to_string()}
                        }
                    }
                }

                // Action
                button {
                    class: "w-full py-3.5 bg-blue-600 hover:bg-blue-700 active:scale-[0.98] text-white rounded-2xl font-semibold shadow-lg shadow-blue-500/25 transition-all",
                    onclick: handle_next,
                    if is_password {
                        {t!("auth.sign_in", locale = l_str).to_string()}
                    } else {
                        {t!("auth.next", locale = l_str).to_string()}
                    }
                }

                // Footer
                div { class: "mt-8 text-center pb-4",
                    p { class: "text-sm text-zinc-500",
                        {t!("auth.no_account", locale = l_str).to_string()} " "
                        Link {
                            to: Route::Register {},
                            class: "font-semibold text-zinc-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 transition-colors ml-1",
                            {t!("auth.sign_up", locale = l_str).to_string()}
                        }
                    }
                }

                // End form container my-auto wrap
                }
            }
        }
        }
    }
}
