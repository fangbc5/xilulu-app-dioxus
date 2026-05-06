//! Xilulu Console — 通用企业管理平台
//!
//! 对话式、命令驱动的下一代管理后台框架。
//! 各微服务通过 ConsoleModule trait 注册为业务模块。

mod components;
mod layout;
mod module;
mod modules;
mod registry;
mod routes;
pub mod theme;
mod views;

use dioxus::prelude::*;
use modules::team::TeamModule;
use registry::build_registry;
use routes::Route;
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

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap",
        }

        // 主题 Provider 包裹整个应用
        ThemeProvider {
            initial: Theme::Jade,
            Router::<Route> {}
        }
    }
}
