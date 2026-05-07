//! 控制台布局
//!
//! 全局布局组件：顶部栏 + 侧边栏 + 主内容区。

use crate::components::command_palette::CommandPaletteProvider;
use crate::components::sidebar::Sidebar;
use crate::registry::RegistryData;
use crate::services::auth::{AuthStage, RefreshTokenRequest, UserInfo};
use crate::services::client::ApiClient;
use crate::services::storage::{clear_auth, save_auth};
use crate::theme::ThemeToggle;
use dioxus::prelude::*;

/// 控制台全局布局
#[component]
pub fn ConsoleLayout() -> Element {
    // 认证守卫：未登录时跳转到 /login
    let mut stage = use_context::<Signal<AuthStage>>();
    let mut access_token = use_context::<Signal<String>>();
    let mut refresh_token_sig = use_context::<Signal<String>>();
    let user_info: Signal<Option<UserInfo>> = use_context();
    let api_client = use_context::<ApiClient>();
    let nav = use_navigator();
    if !matches!(stage(), AuthStage::Authenticated) {
        nav.push("/login");
    }

    // 自动刷新 token：每隔 4 分钟检查一次（假设 token 有效期 5 分钟）
    let api_ref = api_client.clone();
    use_future(move || {
        let api = api_ref.clone();
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(240_000).await;
                let rt = refresh_token_sig();
                if rt.is_empty() {
                    continue;
                }
                match api.auth_refresh_token(&RefreshTokenRequest { refresh_token: rt }).await {
                    Ok(resp) => {
                        api.set_token(&resp.access_token);
                        if !resp.refresh_token.is_empty() {
                            api.set_refresh_token(&resp.refresh_token);
                        }
                        access_token.set(resp.access_token.clone());
                        refresh_token_sig.set(resp.refresh_token.clone());
                        if let Some(ui) = user_info() {
                            save_auth(&access_token(), &refresh_token_sig(), &ui);
                        }
                    }
                    Err(e) => {
                        log::warn!("Token 刷新失败: {e}");
                        // refresh_token 也过期，强制登出
                        if matches!(stage(), AuthStage::Authenticated) {
                            stage.set(AuthStage::Unauthenticated);
                            access_token.set(String::new());
                            refresh_token_sig.set(String::new());
                            clear_auth();
                        }
                    }
                }
            }
        }
    });

    let registry = use_context::<RegistryData>();
    let commands = registry.commands.clone();
    let nav = use_navigator();

    let active_module = registry.modules.first();
    let nav_items = active_module
        .map(|m| registry.nav_items_for(m.id))
        .unwrap_or_default();
    let module_name = active_module.map(|m| m.name).unwrap_or("Console");
    let mut sidebar_collapsed = use_signal(|| false);
    let mut user_menu_open = use_signal(|| false);

    // 获取用户首字母
    let info = user_info();
    let (avatar_char, nickname) = match &info {
        Some(u) => {
            let ch = u.nickname.chars().next().unwrap_or('U').to_uppercase().next().unwrap_or('U');
            (ch.to_string(), u.nickname.clone())
        }
        None => ('?'.to_string(), "未知用户".to_string()),
    };

    rsx! {
        CommandPaletteProvider { commands,

            div { style: "display: flex; flex-direction: column; width: 100%; height: 100%;",

                // === 顶部栏 ===
                header { class: "topbar",

                    // Logo
                    div {
                        style: "display: flex; align-items: center; gap: 10px; cursor: pointer;",
                        onclick: move |_| {
                            let _ = nav.push("/");
                        },
                        span {
                            class: "aurora-text",
                            style: "font-size: 17px; letter-spacing: -0.02em;",
                            "✦ Xilulu"
                        }
                        span { style: "font-size: 12px; color: var(--ds-text-tertiary); font-weight: 500;",
                            "Console"
                        }
                    }

                    // 模块导航
                    nav { style: "display: flex; align-items: center; gap: 2px; margin-left: 40px;",
                        div {
                            style: "padding: 5px 12px; border-radius: 6px; font-size: 13px; font-weight: 500; color: var(--ds-text-secondary); cursor: pointer; transition: all 120ms ease;",
                            onclick: move |_| {
                                let _ = nav.push("/");
                            },
                            "Cockpit"
                        }
                        for module in registry.modules.iter() {
                            {
                                let first_path = registry
                                    .nav_items_for(module.id)
                                    .first()
                                    .map(|n| n.path.clone())
                                    .unwrap_or_else(|| "/".to_string());
                                rsx! {
                                    div {
                                        style: "padding: 5px 12px; border-radius: 6px; font-size: 13px; font-weight: 500; color: var(--ds-text-secondary); cursor: pointer; transition: all 120ms ease;",
                                        onclick: {
                                            let path = first_path.clone();
                                            move |_| {
                                                let _ = nav.push(path.as_str());
                                            }
                                        },
                                        "{module.name}"
                                    }
                                }
                            }
                        }
                    }

                    // 右侧工具
                    div { style: "margin-left: auto; display: flex; align-items: center; gap: 12px; position: relative;",
                        // 主题切换
                        ThemeToggle {}
                        // ⌘K 提示
                        div { class: "cmd-hint", style: "font-size: 12px;",
                            kbd { "⌘K" }
                        }

                        // 用户头像 + 下拉菜单
                        div { style: "position: relative;",
                            div {
                                style: "
                                    display: flex; align-items: center; gap: 8px;
                                    cursor: pointer; padding: 2px 4px; border-radius: 6px;
                                    transition: background 120ms ease;
                                ",
                                onclick: move |_| user_menu_open.toggle(),

                                // 昵称
                                span { style: "font-size: 12px; color: var(--ds-text-secondary); max-width: 80px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                    "{nickname}"
                                }

                                // 头像
                                div { style: "
                                    width: 28px; height: 28px; border-radius: 50%;
                                    background: var(--ds-accent);
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 11px; font-weight: 600; color: var(--ds-text-inverse);
                                ",
                                    "{avatar_char}"
                                }
                            }

                            // 下拉菜单
                            if user_menu_open() {
                                {
                                    rsx! {
                                        // 背景遮罩，点击关闭
                                        div {
                                            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9998;",
                                            onclick: move |_| user_menu_open.set(false),
                                        }

                                        div {
                                            style: "
                                                                                                                                                                                                                                                                                                                                                                                position: absolute; right: 0; top: 40px;
                                                                                                                                                                                                                                                                                                                                                                                width: 160px; padding: 4px;
                                                                                                                                                                                                                                                                                                                                                                                background: var(--ds-bg-secondary);
                                                                                                                                                                                                                                                                                                                                                                                border: 1px solid var(--ds-border-primary);
                                                                                                                                                                                                                                                                                                                                                                                border-radius: 8px;
                                                                                                                                                                                                                                                                                                                                                                                box-shadow: 0 8px 24px rgba(0,0,0,0.12);
                                                                                                                                                                                                                                                                                                                                                                                z-index: 9999;
                                                                                                                                                                                                                                                                                                                                                                            ",

                                            // 用户信息
                                            div {
                                                style: "
                                                                                                                                                                                                                                                                                                                                                                                padding: 8px 12px;
                                                                                                                                                                                                                                                                                                                                                                                border-bottom: 1px solid var(--ds-border-primary);
                                                                                                                                                                                                                                                                                                                                                                                margin-bottom: 4px;
                                                                                                                                                                                                                                                                                                                                                                            ",
                                                div { style: "font-size: 13px; font-weight: 500; color: var(--ds-text-primary);",
                                                    "{nickname}"
                                                }
                                            }

                                            // 注销按钮
                                            button {
                                                style: "
                                                                                                                                                                                                                                                                                                                                                                                    display: block; width: 100%;
                                                                                                                                                                                                                                                                                                                                                                                    padding: 8px 12px; text-align: left;
                                                                                                                                                                                                                                                                                                                                                                                    font-size: 13px; color: var(--ds-text-secondary);
                                                                                                                                                                                                                                                                                                                                                                                    background: none; border: none;
                                                                                                                                                                                                                                                                                                                                                                                    border-radius: 4px; cursor: pointer;
                                                                                                                                                                                                                                                                                                                                                                                    transition: background 100ms ease;
                                                                                                                                                                                                                                                                                                                                                                                ",
                                                onclick: {
                                                    let api = api_client.clone();
                                                    move |_| {
                                                        let api = api.clone();
                                                        spawn(async move {
                                                            let _ = api.auth_logout().await;
                                                        });
                                                        stage.set(AuthStage::Unauthenticated);
                                                        access_token.set(String::new());
                                                        refresh_token_sig.set(String::new());
                                                        clear_auth();
                                                        user_menu_open.set(false);
                                                        nav.push("/login");
                                                    }
                                                },
                                                "退出登录"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // === 主体 ===
                div { style: "display: flex; flex: 1; overflow: hidden;",
                    Sidebar {
                        module_name: module_name.to_string(),
                        items: nav_items,
                        collapsed: sidebar_collapsed(),
                        on_toggle: move |_| sidebar_collapsed.toggle(),
                    }
                    main {
                        class: "main-content",
                        style: "flex: 1; overflow-y: auto; padding: 0 32px 32px; background: var(--ds-bg-primary);",
                        Outlet::<crate::routes::Route> {}
                    }
                }
            }
        }
    }
}