//! 侧边栏组件

use crate::module::NavItem;
use dioxus::prelude::*;

/// 侧边栏
///
/// 根据当前活动模块动态显示导航项。
/// 使用 navigator.push() 进行客户端路由切换（而非 <a> 标签），避免页面刷新。
#[component]
pub fn Sidebar(
    /// 当前模块名称
    module_name: String,
    /// 导航项列表
    items: Vec<NavItem>,
) -> Element {
    let nav = use_navigator();
    let current_path = use_route::<crate::routes::Route>();

    rsx! {
        aside {
            class: "sidebar",
            // 模块标题
            div {
                style: "padding: 20px 16px 12px; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--ds-text-tertiary);",
                "{module_name}"
            }
            // 导航项
            nav {
                style: "flex: 1; display: flex; flex-direction: column;",
                for item in items.iter() {
                    {
                        let path = item.path.clone();
                        let label = item.label.clone();
                        let icon_char = item.icon.clone();
                        let badge = item.badge;
                        // 判断当前路由是否匹配这个导航项
                        let is_active = format!("{}", current_path).starts_with(&path);
                        let active_class = if is_active { " active" } else { "" };
                        rsx! {
                            div {
                                class: "sidebar-item{active_class}",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        let _ = nav.push(path.as_str());
                                    }
                                },
                                // 图标
                                span {
                                    style: "width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; font-size: 14px;",
                                    "{icon_char}"
                                }
                                span { "{label}" }
                                if let Some(b) = badge {
                                    span {
                                        style: "margin-left: auto; background: var(--ds-accent); color: var(--ds-text-inverse); font-size: 11px; padding: 1px 6px; border-radius: 10px; font-weight: 500;",
                                        "{b}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // 底部 ⌘K 提示
            div {
                style: "padding: 16px;",
                div {
                    class: "cmd-hint",
                    kbd { "⌘K" }
                    span { "命令面板" }
                }
            }
        }
    }
}
