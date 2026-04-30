//! 页面标题头组件

use dioxus::prelude::*;

/// 页面标题头
///
/// 统一的页面标题样式，显示标题 + 可选描述。
#[component]
pub fn PageHeader(
    /// 页面标题
    title: String,
    /// 可选描述
    #[props(default)]
    description: Option<String>,
) -> Element {
    rsx! {
        div {
            class: "page-header",
            h1 { "{title}" }
            if let Some(desc) = description {
                p { "{desc}" }
            }
        }
    }
}
