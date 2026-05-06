//! Cockpit 驾驶舱视图
//!
//! 控制台首页，动态聚合所有模块注册的概览卡片，
//! 以及组织心跳、待处理事项、最近活动等模块。

pub mod pulse;
pub mod pending;
pub mod timeline;

pub use pulse::{PulseData, OrganizationPulse};
pub use pending::{PendingItem, PendingType, PendingList};
pub use timeline::{Activity, ActivityType, ActivityTimeline};

use crate::components::stat_card::StatCard;
use crate::registry::RegistryData;
use dioxus::prelude::*;

/// CockpitView 驾驶舱视图
#[component]
pub fn CockpitView() -> Element {
    let registry = use_context::<RegistryData>();
    let cards = registry.cockpit_cards.clone();

    // 模拟脉冲数据（实际应从 API 获取）
    let pulse_data = PulseData {
        total_count: 368,
        trend: 12,
        trend_label: "本周".to_string(),
        vitality: 87,
        trend_percent: 3.2,
    };

    // 模拟待处理数据
    let pending_items = vec![
        PendingItem {
            id: 1,
            r#type: PendingType::NewEmployee,
            title: "3 位新员工等待部门分配".to_string(),
            description: "王五、赵六、孙七".to_string(),
            time: "今天".to_string(),
        },
        PendingItem {
            id: 2,
            r#type: PendingType::DepartmentRequest,
            title: "产品部申请增设\"体验设计组\"".to_string(),
            description: "申请中，等待审批".to_string(),
            time: "昨天".to_string(),
        },
        PendingItem {
            id: 3,
            r#type: PendingType::TransferRequest,
            title: "2 位员工调岗申请待审批".to_string(),
            description: "李四: 产品 > 运营".to_string(),
            time: "2天前".to_string(),
        },
    ];

    // 模拟活动数据
    let activities = vec![
        Activity {
            id: 1,
            r#type: ActivityType::Create,
            actor: "fangbc".to_string(),
            target: "张伟".to_string(),
            target_type: "员工".to_string(),
            time: "10:30".to_string(),
            time_ago: "刚刚".to_string(),
        },
        Activity {
            id: 2,
            r#type: ActivityType::Transfer,
            actor: "admin".to_string(),
            target: "市场部".to_string(),
            target_type: "部门".to_string(),
            time: "09:45".to_string(),
            time_ago: "1小时前".to_string(),
        },
        Activity {
            id: 3,
            r#type: ActivityType::Update,
            actor: "fangbc".to_string(),
            target: "AI实验室".to_string(),
            target_type: "部门".to_string(),
            time: "昨天".to_string(),
            time_ago: "昨天".to_string(),
        },
        Activity {
            id: 4,
            r#type: ActivityType::View,
            actor: "admin".to_string(),
            target: "王五".to_string(),
            target_type: "员工".to_string(),
            time: "昨天".to_string(),
            time_ago: "昨天".to_string(),
        },
    ];

    // 获取问候语
    let greeting = "Good evening";
    let user_name = "fangbc";

    rsx! {
        div { style: "max-width: 1200px; margin: 0 auto; width: 100%;",

            // 问候语
            div {
                class: "cockpit-greeting",
                style: "padding-top: 40px; padding-bottom: 32px;",
                h1 {
                    class: "aurora-text",
                    style: "font-size: 28px; font-weight: 600; letter-spacing: -0.02em; margin-bottom: 4px;",
                    "{greeting}, {user_name}"
                }
                p { style: "color: var(--ds-text-tertiary); font-size: 13.5px;",
                    "Xilulu Console - 管理你的组织和团队"
                }
            }

            // 主要内容网格
            div { style: "display: grid; grid-template-columns: 1fr 380px; gap: 24px;",

                // 左侧：统计卡片 + 活动时间线
                div { style: "display: flex; flex-direction: column; gap: 24px;",

                    // 统计卡片网格
                    if !cards.is_empty() {
                        div {
                            class: "cockpit-cards-grid",
                            style: "grid-template-columns: repeat(2, 1fr);",
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

                    // 最近活动时间线
                    div { class: "glass-card",
                        ActivityTimeline { activities: activities.clone() }
                    }
                }

                // 右侧：组织心跳 + 待处理
                div { style: "display: flex; flex-direction: column; gap: 24px;",

                    // 组织心跳
                    div { class: "glass-card",
                        OrganizationPulse { data: pulse_data.clone() }
                    }

                    // 待处理事项
                    div { class: "glass-card",
                        PendingList {
                            items: pending_items.clone(),
                            on_item_click: |_id: u64| {},
                        }
                    }
                }
            }

            // ⌘K 引导
            div { style: "display: flex; justify-content: center; padding: 40px 0; animation: page-enter 500ms cubic-bezier(0.16, 1, 0.3, 1) 300ms both;",
                div {
                    class: "glass-card",
                    style: "padding: 20px 32px; display: flex; align-items: center; gap: 16px;",
                    div { style: "font-size: 13px; color: var(--ds-text-secondary);",
                        "试试说点什么"
                    }
                    div { class: "cmd-hint",
                        kbd { "Cmd K" }
                    }
                }
            }
        }
    }
}
