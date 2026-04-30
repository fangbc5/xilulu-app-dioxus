//! 控制台布局
//!
//! 全局布局组件：顶部栏 + 侧边栏 + 主内容区。

use crate::components::command_palette::CommandPaletteProvider;
use crate::components::sidebar::Sidebar;
use crate::registry::RegistryData;
use crate::theme::ThemeToggle;
use dioxus::prelude::*;

/// 控制台全局布局
#[component]
pub fn ConsoleLayout() -> Element {
    let registry = use_context::<RegistryData>();
    let commands = registry.commands.clone();
    let nav = use_navigator();

    let active_module = registry.modules.first();
    let nav_items = active_module
        .map(|m| registry.nav_items_for(m.id))
        .unwrap_or_default();
    let module_name = active_module.map(|m| m.name).unwrap_or("Console");

    rsx! {
        CommandPaletteProvider {
            commands: commands,

            div {
                style: "display: flex; flex-direction: column; width: 100%; height: 100%;",

                // === 顶部栏 ===
                header {
                    class: "topbar",

                    // Logo
                    div {
                        style: "display: flex; align-items: center; gap: 10px; cursor: pointer;",
                        onclick: move |_| { let _ = nav.push("/"); },
                        span {
                            class: "aurora-text",
                            style: "font-size: 17px; letter-spacing: -0.02em;",
                            "✦ Xilulu"
                        }
                        span {
                            style: "font-size: 12px; color: var(--ds-text-tertiary); font-weight: 500;",
                            "Console"
                        }
                    }

                    // 模块导航（使用 onclick + navigator 代替 <a> 标签）
                    nav {
                        style: "display: flex; align-items: center; gap: 2px; margin-left: 40px;",
                        div {
                            style: "padding: 5px 12px; border-radius: 6px; font-size: 13px; font-weight: 500; color: var(--ds-text-secondary); cursor: pointer; transition: all 120ms ease;",
                            onclick: move |_| { let _ = nav.push("/"); },
                            "Cockpit"
                        }
                        for module in registry.modules.iter() {
                            {
                                let first_path = registry.nav_items_for(module.id)
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
                    div {
                        style: "margin-left: auto; display: flex; align-items: center; gap: 12px;",
                        // 主题切换
                        ThemeToggle {}
                        // ⌘K 提示
                        div {
                            class: "cmd-hint",
                            style: "font-size: 12px;",
                            kbd { "⌘K" }
                        }
                        // 用户头像
                        div {
                            style: "width: 30px; height: 30px; border-radius: 50%; background: var(--ds-accent); display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 600; color: var(--ds-text-inverse);",
                            "F"
                        }
                    }
                }

                // === 主体 ===
                div {
                    style: "display: flex; flex: 1; overflow: hidden;",
                    Sidebar {
                        module_name: module_name.to_string(),
                        items: nav_items,
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
