use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdChevronLeft;
use crate::i18n::use_language;
use rust_i18n::t;

#[component]
pub fn Terms() -> Element {
    let nav = use_navigator();
    let lang = use_language();
    let l_str = lang().as_str();

    rsx! {
        div { class: "min-h-screen bg-zinc-50 dark:bg-[#0A0A0B] text-zinc-900 dark:text-zinc-100 p-6 sm:p-12",
            div { class: "max-w-3xl mx-auto",
                div { class: "flex items-center gap-4 mb-12",
                    button {
                        class: "w-10 h-10 rounded-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 flex items-center justify-center hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors",
                        onclick: move |_| { nav.go_back(); },
                        Icon { icon: LdChevronLeft, width: 20, height: 20 }
                    }
                    h1 { class: "text-3xl font-bold tracking-tight", {t!("docs.terms_title", locale = l_str).to_string()} }
                }
                
                div { class: "prose dark:prose-invert max-w-none space-y-6",
                    p { class: "text-zinc-500 dark:text-zinc-400 font-medium", {t!("docs.last_updated", locale = l_str).to_string()} }
                    
                    div { class: "bg-white dark:bg-zinc-800/50 rounded-2xl p-6 sm:p-8 border border-zinc-200 dark:border-zinc-800 shadow-sm",
                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h1", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed mb-8", {t!("docs.terms_p1", locale = l_str).to_string()} }
                        
                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h2", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed mb-8", {t!("docs.terms_p2", locale = l_str).to_string()} }

                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h3", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed mb-8", {t!("docs.terms_p3", locale = l_str).to_string()} }

                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h4", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed mb-8", {t!("docs.terms_p4", locale = l_str).to_string()} }

                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h5", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed mb-8", {t!("docs.terms_p5", locale = l_str).to_string()} }

                        h2 { class: "text-xl font-bold mb-4 text-zinc-900 dark:text-white flex items-center gap-2", 
                            span { class: "text-blue-600 dark:text-blue-400", "#" }
                            {t!("docs.terms_h6", locale = l_str).to_string()} 
                        }
                        p { class: "text-zinc-600 dark:text-zinc-300 leading-relaxed", {t!("docs.terms_p6", locale = l_str).to_string()} }
                    }
                }
            }
        }
    }
}
