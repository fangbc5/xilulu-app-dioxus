//! 毛玻璃卡片组件

use dioxus::prelude::*;

/// 毛玻璃卡片容器
///
/// 深空主题核心组件，提供半透明背景 + 模糊 + 微光边框效果。
#[component]
pub fn GlassCard(
    /// 额外的 CSS class
    #[props(default = String::new())]
    class: String,
    /// 子元素
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "glass-card {class}",
            {children}
        }
    }
}
