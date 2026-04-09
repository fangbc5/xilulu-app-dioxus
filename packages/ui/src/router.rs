use dioxus::prelude::*;

use crate::pages::auth::login::Login;
use crate::pages::auth::register::Register;
use crate::pages::auth::otp::OtpVerify;
use crate::pages::auth::setup::ProfileSetup;
use crate::pages::docs::terms::Terms;
use crate::pages::docs::privacy::Privacy;
use crate::i18n::{use_i18n_provider, use_language};
use rust_i18n::t;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[redirect("/", || Route::Login {})]
    #[route("/auth/login")]
    Login {},

    #[route("/auth/register")]
    Register {},

    #[route("/auth/otp/:medium")]
    OtpVerify { medium: String },

    #[route("/auth/setup")]
    ProfileSetup {},

    #[route("/docs/terms")]
    Terms {},

    #[route("/docs/privacy")]
    Privacy {},

    #[route("/app")]
    AppHome {},
}

#[component]
pub fn AppRoot() -> Element {
    // Initialize I18n globally
    use_i18n_provider();
    
    rsx! {
        Router::<Route> {}
    }
}

#[component]
pub fn AppHome() -> Element {
    let lang = use_language();
    let l_str = lang().as_str();
    #[cfg(feature = "desktop")]
    {
        use_effect(move || {
            let window = dioxus::desktop::window();
            window.set_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0));
            window.set_resizable(true);
            window.set_maximizable(true);

            if let Some(monitor) = window.current_monitor() {
                let screen_size = monitor.size();
                let scale = monitor.scale_factor();
                let screen_logical_w = screen_size.width as f64 / scale;
                let screen_logical_h = screen_size.height as f64 / scale;
                let x = (screen_logical_w - 1200.0) / 2.0;
                let y = (screen_logical_h - 800.0) / 2.0;
                window.set_outer_position(dioxus::desktop::LogicalPosition::new(x, y));
            }
        });
    }

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-50 dark:bg-zinc-900",
            div {
                class: "text-center p-8 bg-white dark:bg-zinc-800 rounded-3xl shadow-2xl transition-all",
                h1 { class: "text-4xl font-bold text-zinc-900 dark:text-white mb-4", {t!("auth.app_home", locale=l_str).to_string()} }
                p { class: "text-zinc-500", {t!("auth.app_home_desc", locale=l_str).to_string()} }
                
                Link {
                    to: Route::Login {},
                    class: "mt-8 inline-block px-6 py-2 rounded-full bg-zinc-100 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-200 hover:bg-zinc-200 dark:hover:bg-zinc-600 transition",
                    {t!("auth.logout", locale=l_str).to_string()}
                }
            }
        }
    }
}
