//! Xilulu Console — 通用企业管理平台
//!
//! 对话式、命令驱动的下一代管理后台框架。
//! 各微服务通过 ConsoleModule trait 注册为业务模块。

mod components;
mod conversation;
mod executor;
mod intent;
mod layout;
mod module;
mod modules;
mod registry;
mod routes;
pub mod services;
pub mod skill;
pub mod theme;
mod views;

use dioxus::prelude::*;
use modules::team::TeamModule;
use registry::build_registry;
use routes::Route;
use services::auth::{AuthStage, UserInfo};
use services::client::ApiClient;
use services::storage::restore_auth;
use theme::{Theme, ThemeProvider};

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

/// 根组件
#[component]
fn App() -> Element {
    // 构建模块注册数据
    let registry_data = use_hook(|| {
        let modules: Vec<Box<dyn module::ConsoleModule>> = vec![
            Box::new(TeamModule),
        ];
        build_registry(modules)
    });

    use_context_provider(|| registry_data);

    // 从 localStorage 恢复认证状态
    let (init_stage, init_access, init_refresh, init_user_info) = use_hook(|| {
        let (access, refresh, user) = restore_auth();
        if access.is_empty() || refresh.is_empty() {
            (AuthStage::Unauthenticated, String::new(), String::new(), None::<UserInfo>)
        } else {
            (AuthStage::Authenticated, access, refresh, user)
        }
    });

    // 全局 API 客户端
    let api_client = use_hook(|| {
        let mut client = ApiClient::new("http://localhost:8080");
        if !init_access.is_empty() {
            client.set_token(&init_access);
        }
        client
    });
    use_context_provider(|| api_client);

    // 认证状态
    use_context_provider(|| Signal::new(init_stage));
    use_context_provider(|| Signal::new(init_access));
    use_context_provider(|| Signal::new(init_refresh));
    use_context_provider(|| Signal::new(init_user_info));

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap",
        }

        // 主题 Provider 包裹整个应用
        ThemeProvider { initial: Theme::Jade, Router::<Route> {} }
    }
}