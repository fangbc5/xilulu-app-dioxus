use super::Route;
use crate::components::feedback::ToastProvider;
use crate::i18n::use_i18n_provider;
use dioxus::prelude::*;

/// 全局应用根组件（所有平台共享）
/// 负责初始化 i18n、Session、Toast，并挂载对应平台的 Router
#[component]
pub fn AppRoot() -> Element {
    use_i18n_provider();

    // 初始化外挂业务层 Core Crate
    let core_app_opt = use_resource(move || async move {
        // Desktop 这里直接写入 Adapter 实例，如果是 Mobile 可以根据条件编译加载其他的
        let storage = std::sync::Arc::new(crate::store::adapter::DesktopStorageAdapter);
        let core_app = xilulu_im_sdk::service::app::CoreApp::new(storage).await;
        std::sync::Arc::new(core_app)
    });

    let core_app = match core_app_opt.read().as_ref() {
        Some(app) => app.clone(),
        None => {
            return rsx! {
                div { class: "w-full h-full flex items-center justify-center bg-white dark:bg-black",
                    "Loading Core Systems..."
                }
            };
        }
    };

    // 注入唯一的全知服务端点
    use_context_provider(|| core_app.clone());

    rsx! {
        ToastProvider { Router::<Route> {} }
    }
}

pub fn use_auth_guard() {
    let nav = use_navigator();
    let core_app = use_context::<std::sync::Arc<xilulu_im_sdk::service::app::CoreApp>>();
    let session = core_app.auth.current_session();

    // 可以在这里监听 auth 是否因为 HTTP 失败主动退出，如果退出了则重定向
    use_effect(move || {
        if !session.is_authenticated() {
            // Dioxus 无法直接捕捉静态全局 AUTH_FAILED，
            // 我们可以靠对核心响应式抛出的 session 状态去判断
            // 稍后在 API 调用失败后自动清除 session 时即可触发此处跳转！
            nav.replace(Route::Login {});
        }
    });
}
