//! 统计数字卡片组件

use dioxus::prelude::*;

/// 统计数字卡片
///
/// 在驾驶舱首页展示数字概览，毛玻璃底 + 大字数值 + 趋势箭头。
#[component]
pub fn StatCard(
    /// 标签文字（如 "在职员工"）
    label: String,
    /// 数值（如 "368"）
    value: String,
    /// 趋势文字（如 "△12 本周"）
    #[props(default)]
    trend: Option<String>,
    /// 趋势是否正向（绿色/红色）
    #[props(default = true)]
    trend_positive: bool,
) -> Element {
    let trend_class = if trend_positive { "positive" } else { "negative" };

    rsx! {
        div {
            class: "glass-card stat-card",
            div { class: "stat-label", "{label}" }
            div {
                class: "stat-value",
                style: "animation: number-appear 600ms ease-out",
                "{value}"
            }
            if let Some(trend_text) = trend {
                div {
                    class: "stat-trend {trend_class}",
                    "{trend_text}"
                }
            }
        }
    }
}
