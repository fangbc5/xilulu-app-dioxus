//! 驾驶舱首页
//!
//! Console 的默认首页，动态聚合所有模块注册的概览卡片。

use crate::components::stat_card::StatCard;
use crate::registry::RegistryData;
use dioxus::prelude::*;

/// 驾驶舱视图
#[component]
pub fn CockpitView() -> Element {
    let registry = use_context::<RegistryData>();
    let cards = registry.cockpit_cards.clone();

    rsx! {
        div {
            style: "max-width: 900px; margin: 0 auto; width: 100%;",

            // 问候语
            div {
                class: "cockpit-greeting",
                style: "padding-top: 40px; padding-bottom: 28px;",
                h1 {
                    class: "aurora-text",
                    style: "font-size: 28px; font-weight: 600; letter-spacing: -0.02em; margin-bottom: 4px;",
                    "Welcome back ✦"
                }
                p {
                    style: "color: var(--ds-text-tertiary); font-size: 13.5px;",
                    "Xilulu Console · 管理你的组织和团队"
                }
            }

            // 统计卡片网格（交错入场动画）
            if !cards.is_empty() {
                div {
                    class: "cockpit-cards-grid",
                    for card in cards.iter() {
                        StatCard {
                            label: card.title.clone(),
                            value: card.value.clone(),
                            trend: card.trend.clone(),
                            trend_positive: card.trend_positive,
                        }
                    }
                }
            }

            // ⌘K 引导
            div {
                style: "display: flex; flex-direction: column; align-items: center; padding: 40px 0; animation: page-enter 500ms cubic-bezier(0.16, 1, 0.3, 1) 300ms both;",
                div {
                    class: "glass-card",
                    style: "padding: 28px 40px; text-align: center; max-width: 420px;",
                    div {
                        style: "font-size: 13.5px; color: var(--ds-text-secondary); margin-bottom: 14px;",
                        "快速开始"
                    }
                    div {
                        class: "cmd-hint",
                        style: "font-size: 13px;",
                        kbd { "⌘K" }
                        span { "打开命令面板" }
                    }
                    div {
                        style: "font-size: 12px; color: var(--ds-text-tertiary); margin-top: 14px; line-height: 1.6;",
                        "输入 \"员工\" 查看员工管理 · 输入 \"部门\" 查看部门树"
                    }
                }
            }
        }
    }
}
