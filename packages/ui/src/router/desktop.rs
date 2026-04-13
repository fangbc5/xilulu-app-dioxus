use dioxus::prelude::*;

use crate::pages::auth::login::Login;
use crate::pages::auth::otp::OtpVerify;
use crate::pages::auth::register::Register;
use crate::pages::auth::setup::ProfileSetup;
use crate::pages::docs::privacy::Privacy;
use crate::pages::docs::terms::Terms;

const CLIENT_APP_ID: i32 = 3; // xilulu-desktop

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    // ======= 共享认证路由 =======
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

    // ======= Desktop 业务路由 =======
    #[redirect("/app", || Route::AppDashboard {})]
    #[layout(DesktopLayout)]
    #[route("/app/dashboard")]
    AppDashboard {},

    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// Desktop 端主布局：侧边栏导航（暂占位）
#[component]
fn DesktopLayout() -> Element {
    super::common::use_auth_guard();

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
        div { class: "flex min-h-screen bg-zinc-100 dark:bg-zinc-900",
            // 侧边栏占位（后续扩展）
            aside { class: "w-64 bg-white dark:bg-zinc-800 border-r border-zinc-200 dark:border-zinc-700 hidden lg:block p-4",
                h2 { class: "text-lg font-bold text-zinc-900 dark:text-white mb-4",
                    "Xilulu"
                }
                nav { class: "space-y-2",
                    Link {
                        to: Route::AppDashboard {},
                        class: "block px-4 py-2 rounded-lg text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-700",
                        "Dashboard"
                    }
                }
            }
            // 主内容区
            main { class: "flex-1", Outlet::<Route> {} }
        }
    }
}

/// Desktop 主面板占位页
#[component]
fn AppDashboard() -> Element {
    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center p-8",
                h1 { class: "text-3xl font-bold text-zinc-900 dark:text-white mb-4",
                    "Desktop Dashboard"
                }
                p { class: "text-zinc-500", "桌面端工作台 - 待开发" }
            }
        }
    }
}

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    rsx! { div { "404 Not Found" } }
}
