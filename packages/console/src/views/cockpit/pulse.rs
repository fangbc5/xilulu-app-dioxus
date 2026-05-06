//! 组织心跳 (Organization Pulse)
//!
//! 实时展示组织状态的核心组件，带呼吸动画。

use dioxus::prelude::*;

/// 组织心跳数据
#[derive(Clone, Debug, PartialEq)]
pub struct PulseData {
    pub total_count: u32,
    pub trend: i32,
    pub trend_label: String,
    pub vitality: u8,
    pub trend_percent: f32,
}

/// 组织心跳组件
///
/// 显示在职人数、趋势、活力指数，带脉冲呼吸动画。
#[component]
pub fn OrganizationPulse(
    data: PulseData,
) -> Element {
    let dots = (data.vitality / 10) as usize;
    let trend_icon = if data.trend >= 0 { "+" } else { "-" };
    let trend_color_attr = if data.trend >= 0 {
        "color: var(--ds-success);"
    } else {
        "color: var(--ds-error);"
    };

    // 生成活性点
    let dot_colors: Vec<&str> = (0..10)
        .map(|i| if i < dots { "var(--ds-accent)" } else { "var(--ds-border)" })
        .collect();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 16px; padding: 24px;",

            // 头部统计
            div {
                style: "display: flex; align-items: flex-end; gap: 16px;",
                // 人数
                div {
                    div {
                        style: "font-size: 13px; color: var(--ds-text-secondary); font-weight: 500; margin-bottom: 4px;",
                        "在职人数"
                    }
                    div {
                        style: "display: flex; align-items: baseline; gap: 10px;",
                        span {
                            class: "aurora-text",
                            style: "font-size: 48px; font-weight: 700; letter-spacing: -0.03em; line-height: 1;",
                            "{data.total_count}"
                        }
                        span {
                            style: "font-size: 14px; color: var(--ds-text-tertiary);",
                            "人"
                        }
                    }
                }

                // 趋势
                div {
                    style: "margin-left: auto; display: flex; flex-direction: column; align-items: flex-end; gap: 2px;",
                    div {
                        style: "font-size: 20px; font-weight: 600; {trend_color_attr}",
                        "{trend_icon}{data.trend.abs()}"
                    }
                    div {
                        style: "font-size: 12px; color: var(--ds-text-tertiary);",
                        "{data.trend_label}"
                    }
                }
            }

            // 活力指示器
            div {
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    span {
                        style: "font-size: 12px; color: var(--ds-text-secondary); font-weight: 500;",
                        "活力指数"
                    }
                    span {
                        style: "font-size: 13px; font-weight: 600; color: var(--ds-accent);",
                        "{data.vitality}%"
                    }
                }
                // 进度条
                div {
                    style: "height: 6px; background: var(--ds-bg-surface); border-radius: 3px; overflow: hidden; position: relative;",
                    div {
                        style: "height: 100%; width: {data.vitality}%; background: linear-gradient(90deg, var(--ds-accent), var(--ds-accent-light)); border-radius: 3px; position: relative;",
                        // 发光脉冲
                        div {
                            style: "position: absolute; right: 0; top: 50%; transform: translateY(-50%); width: 12px; height: 12px; background: var(--ds-accent-light); border-radius: 50%; box-shadow: 0 0 12px var(--ds-accent); animation: pulse-glow 2s ease-in-out infinite;"
                        }
                    }
                }
                // 趋势文字
                div {
                    style: "display: flex; justify-content: space-between; margin-top: 6px;",
                    span {
                        style: "font-size: 11px; color: var(--ds-text-tertiary);",
                        "{data.trend_percent.abs()}%"
                    }
                    span {
                        style: "font-size: 11px; {trend_color_attr}",
                        "{trend_icon} {data.trend_percent.abs()}% vs 上周"
                    }
                }
            }

            // 活性点
            div {
                style: "display: flex; gap: 4px; align-items: center;",
                for (i, color) in dot_colors.iter().enumerate() {
                    span {
                        key: "{i}",
                        style: "width: 8px; height: 8px; border-radius: 50%; background: {color}; transition: background 0.3s ease;"
                    }
                }
                span {
                    style: "font-size: 11px; color: var(--ds-text-tertiary); margin-left: 6px;",
                    "活性度"
                }
            }
        }
    }
}
