use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdCheck, LdInfo, LdX};
use dioxus_free_icons::Icon;
use std::sync::atomic::{AtomicU64, Ordering};

static TOAST_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
    pub expires_at: u64,
}

#[derive(Clone, Copy)]
pub struct ToastManager {
    toasts: Signal<Vec<Toast>>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Signal::new(Vec::new()),
        }
    }

    pub fn show(&self, message: impl Into<String>, variant: ToastVariant) {
        let msg = message.into();
        let id = TOAST_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let toast = Toast {
            id,
            message: msg,
            variant,
            expires_at: now + 3000,
        };
        let mut t = self.toasts;
        t.write().push(toast);
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Success);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Error);
    }

    pub fn info(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Info);
    }

    pub fn remove(&self, id: u64) {
        let mut t = self.toasts;
        t.write().retain(|t| t.id != id);
    }
}

pub fn use_toast() -> ToastManager {
    try_use_context::<ToastManager>().unwrap_or_else(|| {
        tracing::warn!(
            "ToastManager context missing! Creating a local one (won't render globally)."
        );
        ToastManager::new()
    })
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let manager = use_context_provider(|| ToastManager::new());
    let mut toasts = manager.toasts;

    use_future(move || async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let has_expired = toasts.read().iter().any(|t| t.expires_at <= now);
            if has_expired {
                toasts.write().retain(|t| t.expires_at > now);
            }
        }
    });

    rsx! {
        {children}
        // Toast Container
        div { 
            class: "fixed left-1/2 -translate-x-1/2 z-[200] flex flex-col gap-2 pointer-events-none items-center w-full px-4",
            style: "top: max(1rem, env(safe-area-inset-top));",
            for toast in toasts().into_iter() {
                div {
                    key: "{toast.id}",
                    class: "pointer-events-auto flex items-center gap-3 px-4 py-3 bg-white dark:bg-zinc-800 rounded-xl shadow-[0_4px_20px_rgba(0,0,0,0.08)] border border-zinc-100 dark:border-zinc-700/50 max-w-sm transition-all animate-in fade-in slide-in-from-top-4 duration-300",
                    match toast.variant {
                        ToastVariant::Success => rsx! {
                            Icon {
                                icon: LdCheck,
                                width: 18,
                                height: 18,
                                class: "text-emerald-500 shrink-0",
                            }
                        },
                        ToastVariant::Error => rsx! {
                            Icon {
                                icon: LdX,
                                width: 18,
                                height: 18,
                                class: "text-red-500 shrink-0",
                            }
                        },
                        ToastVariant::Info => rsx! {
                            Icon {
                                icon: LdInfo,
                                width: 18,
                                height: 18,
                                class: "text-blue-500 shrink-0",
                            }
                        },
                    }
                    span { class: "text-[14px] font-medium text-zinc-700 dark:text-zinc-200",
                        "{toast.message}"
                    }
                }
            }
        }
    }
}
