use dioxus::prelude::*;

use crate::components::layout::bottom_nav::{BottomNav, BottomNavItem};
use crate::pages::app::chat::AppChat;
use crate::pages::app::contact::AppContact;
use crate::pages::app::social::AppSocial;
use crate::pages::app::user::AppUser;
use crate::pages::auth::login::Login;
use crate::pages::auth::otp::OtpVerify;
use crate::pages::auth::register::Register;
use crate::pages::auth::setup::ProfileSetup;
use crate::pages::docs::privacy::Privacy;
use crate::pages::docs::terms::Terms;

const CLIENT_APP_ID: i32 = 2; // xilulu-mobile

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

    // ======= Mobile 业务路由 =======
    #[redirect("/app", || Route::MobileMain { tab: "chat".into() })]
    #[route("/app/:tab")]
    MobileMain { tab: String },

    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

fn parse_server_path(path: &str) -> Route {
    let tab = match path {
        "/app/chat" | "chat" => "chat",
        "/app/contact" | "contact" => "contact",
        "/app/social" | "social" => "social",
        "/app/user" | "user" => "user",
        _ => "chat",
    };
    Route::MobileMain {
        tab: tab.to_string(),
    }
}

/// Mobile 端主界面：Persistent Tab Navigator
#[component]
pub fn MobileMain(tab: String) -> Element {
    super::common::use_auth_guard();

    let core_app = use_context::<std::sync::Arc<xilulu_im_sdk::service::app::CoreApp>>();
    let api_client = core_app.api_client.clone();

    let menus_resource = use_resource(move || {
        let api = api_client.clone();
        async move {
            match xilulu_im_sdk::api::identity::get_user_menus(&api, CLIENT_APP_ID, None).await {
                Ok(data) => {
                    let menus: Vec<xilulu_im_sdk::api::identity::ResourceInfo> = data;
                    menus
                        .into_iter()
                        .filter(|itm| itm.resource_type.as_deref() == Some("20"))
                        .map(|item| BottomNavItem {
                            target_route: item
                                .path
                                .as_ref()
                                .map(|p| parse_server_path(p))
                                .unwrap_or(Route::MobileMain { tab: "chat".into() }),
                            icon: item.icon.unwrap_or_default(),
                            label: item.name,
                        })
                        .collect::<Vec<BottomNavItem>>()
                }
                Err(e) => {
                    tracing::error!("MobileMain 获取菜单失败: {:?}", e);
                    vec![]
                }
            }
        }
    });

    let nav_items = menus_resource.read().clone();

    // 核心骨架优化：固定高宽，利用 flex-1 min-h-0 relative 完全锁死溢出撑爆问题
    rsx! {
        div { class: "flex flex-col overflow-hidden bg-white dark:bg-[#111111] relative",
            // Tab 容器区域 (将 CSS 黑魔法封装于 TabPanel，实现业务级完全解耦)
            div { class: "flex-1 min-h-0 relative",
                TabPanel { is_active: tab == "chat", AppChat {} }
                TabPanel { is_active: tab == "contact", AppContact {} }
                TabPanel { is_active: tab == "social", AppSocial {} }
                TabPanel { is_active: tab == "user", AppUser {} }
            }
            // 底部菜单，通过固定结构放置在最下方
            BottomNav { items: nav_items }
        }
    }
}

/// 纯粹的视图包裹器 (View Wrapper Container)
/// 用于对抗目前 Dioxus 依赖原生 iOS/Android WebView 渲染时遭遇的 `display: none` 状态丢失怪癖。
/// 未来如果 Dioxus 进化为 Skia/Native UI 直接渲染引擎 (完全脱离 WebView HTML)，
/// 只需要重构或废弃这个组件，您的 4 大核心业务页 (`AppChat` 等) 代码可以 100% 无痛直接迁移。
#[component]
fn TabPanel(is_active: bool, children: Element) -> Element {
    let classes = if is_active {
        "absolute inset-0 z-10 opacity-100 transition-opacity duration-150"
    } else {
        "absolute inset-0 z-0 opacity-0 pointer-events-none"
    };

    rsx! {
        div { class: classes, {children} }
    }
}

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div { "404 Not Found" }
    }
}
