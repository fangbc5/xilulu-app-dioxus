//! 错误页面
//!
//! 404 和 500 友好错误页面。

use dioxus::prelude::*;

/// 404 页面
#[component]
pub fn NotFoundPage(route: Vec<String>) -> Element {
    let nav = use_navigator();
    let path = route.join("/");

    rsx! {
        div { style: "
                display: flex; align-items: center; justify-content: center;
                min-height: 100vh; width: 100%;
                background: var(--ds-bg-primary);
            ",

            div { style: "
                    text-align: center;
                    display: flex; flex-direction: column; align-items: center; gap: 20px;
                ",

                // 404 大字
                div { style: "
                        font-size: 96px; font-weight: 700;
                        background: linear-gradient(135deg, #6366f1, #8b5cf6, #a855f7);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        line-height: 1;
                    ",
                    "404"
                }

                h2 { style: "font-size: 20px; font-weight: 500; color: var(--ds-text-primary);",
                    "页面未找到"
                }

                if !path.is_empty() {
                    p { style: "font-size: 14px; color: var(--ds-text-tertiary);",
                        "路径 /{path} 不存在"
                    }
                }

                p { style: "font-size: 13px; color: var(--ds-text-tertiary); max-width: 360px;",
                    "你访问的页面可能已被移动或删除，请检查地址是否正确。"
                }

                // 返回首页按钮
                button {
                    style: "
                        padding: 10px 24px;
                        background: var(--ds-accent);
                        color: white; border: none;
                        border-radius: 8px; font-size: 14px;
                        font-weight: 500; cursor: pointer;
                        transition: opacity 150ms ease;
                    ",
                    onclick: move |_| {
                        let _ = nav.push("/");
                    },
                    "返回首页"
                }
            }
        }
    }
}