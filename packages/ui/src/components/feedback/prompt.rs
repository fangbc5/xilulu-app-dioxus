use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PromptProps {
    // The bool controls visibility. Can be managed by caller.
    pub show: Signal<bool>,
    pub title: String,
    pub description: String,
    pub cancel_text: String,
    pub confirm_text: String,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

#[component]
pub fn Prompt(props: PromptProps) -> Element {
    if !(props.show)() {
        return rsx! { "" };
    }

    rsx! {
        div { class: "fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm",
            div { class: "bg-white dark:bg-zinc-900 rounded-2xl w-full max-w-sm overflow-hidden shadow-2xl border border-zinc-200 dark:border-zinc-800",
                div { class: "p-6",
                    h3 { class: "text-xl font-bold mb-3 text-zinc-900 dark:text-white",
                        "{props.title}"
                    }
                    p { class: "text-zinc-500 dark:text-zinc-400 text-sm mb-6", "{props.description}" }
                    div { class: "flex gap-3",
                        button {
                            class: "flex-1 py-2.5 rounded-xl text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 font-medium transition-colors",
                            onclick: move |_| {
                                props.on_cancel.call(());
                            },
                            "{props.cancel_text}"
                        }
                        button {
                            class: "flex-1 py-2.5 rounded-xl text-white bg-blue-600 hover:bg-blue-700 font-medium transition-colors shadow-lg shadow-blue-500/25",
                            onclick: move |_| {
                                props.on_confirm.call(());
                            },
                            "{props.confirm_text}"
                        }
                    }
                }
            }
        }
    }
}
