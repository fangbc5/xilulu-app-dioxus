//! 侧边栏组件

use crate::module::NavItem;
use dioxus::prelude::*;

/// 侧边栏
///
/// 根据当前活动模块动态显示导航项。
/// 支持展开/收起模式，按钮仿照 ChatGPT 风格。
#[component]
pub fn Sidebar(
    /// 当前模块名称
    module_name: String,
    /// 导航项列表
    items: Vec<NavItem>,
    /// 是否收起
    collapsed: bool,
    /// 收起/展开回调
    on_toggle: EventHandler<()>,
) -> Element {
    let nav = use_navigator();
    let current_path = use_route::<crate::routes::Route>();

    let sidebar_class = if collapsed {
        "sidebar sidebar-collapsed"
    } else {
        "sidebar"
    };

    let arrow_rotation = if collapsed { 0 } else { 180 };
    let toggle_title = if collapsed { "展开侧栏" } else { "收起侧栏" };

    rsx! {
        // sidebar-area 作为相对定位容器，按钮用 absolute 浮在外面
        div { class: "sidebar-area",
            aside { class: "{sidebar_class}",
                // 模块标题
                div { class: "sidebar-title", "{module_name}" }

                // 导航项
                nav { class: "sidebar-nav",
                    for item in items.iter() {
                        {
                            let path = item.path.clone();
                            let label = item.label.clone();
                            let icon_char = item.icon.clone();
                            let badge = item.badge;
                            let current_str = format!("{}", current_path);
                            let is_active = if path == "/" {
                                current_str == "/"
                            } else {
                                current_str.starts_with(&path)
                            };
                            let active_class = if is_active { " active" } else { "" };
                            rsx! {
                                div {
                                    class: "sidebar-item{active_class}",
                                    title: "{label}",
                                    onclick: {
                                        let path = path.clone();
                                        move |_| {
                                            let _ = nav.push(path.as_str());
                                        }
                                    },
                                    span { class: "sidebar-item-icon", "{icon_char}" }
                                    span { class: "sidebar-item-label", "{label}" }
                                    if let Some(b) = badge {
                                        span { class: "sidebar-item-badge", "{b}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 收起/展开按钮 — 定位在 sidebar-area 内，不受 sidebar 宽度影响
            button {
                class: "sidebar-toggle-btn",
                r#type: "button",
                onclick: move |_| on_toggle.call(()),
                title: "{toggle_title}",
                svg {
                    width: "16",
                    height: "16",
                    view_box: "0 0 16 16",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.8",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    style: "transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1); transform: rotate({arrow_rotation}deg);",
                    polyline { points: "6,3 11,8 6,13" }
                }
            }
        }
    }
}