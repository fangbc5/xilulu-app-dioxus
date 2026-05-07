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
use crate::module::CockpitCard;
use crate::registry::RegistryData;
use crate::services::auth::UserInfo;
use crate::services::client::ApiClient;
use dioxus::prelude::*;

/// 根据当前小时数返回问候语
fn get_greeting() -> &'static str {
    let hour = js_sys::Date::new(&0.into()).get_hours() as u8;
    match hour {
        0..=5 => "夜深了",
        6..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "夜深了",
    }
}

/// CockpitView 驾驶舱视图
#[component]
pub fn CockpitView() -> Element {
    let registry = use_context::<RegistryData>();

    // 从 context 获取当前用户信息
    let user_info: Signal<Option<UserInfo>> = use_context();
    let user_name = user_info()
        .map(|u| u.nickname.clone())
        .unwrap_or_else(|| "用户".to_string());
    let greeting = get_greeting();

    // 从 API 获取统计数据
    let api_client = use_context::<ApiClient>();
    let stats_resource = use_resource(move || {
        let client = api_client.clone();
        async move {
            let emp_count = client.count_employees(1).await.unwrap_or(0);
            let dept_count = client.count_departments(1).await.unwrap_or(0);
            (emp_count, dept_count)
        }
    });

    let (employee_count, department_count) = stats_resource()
        .map(|(e, d)| (e, d))
        .unwrap_or((0, 0));

    // 用真实 API 数据覆盖模块注册卡片中的硬编码值
    let cards: Vec<CockpitCard> = registry.cockpit_cards.iter().map(|card| {
        let mut c = card.clone();
        match c.title.as_str() {
            "在职员工" => {
                c.value = employee_count.to_string();
                c.trend = if employee_count > 0 { None } else { c.trend.take() };
            }
            "部门数量" => {
                c.value = department_count.to_string();
                c.trend = None;
            }
            _ => {}
        }
        c
    }).collect();

    let pulse_data = PulseData {
        total_count: employee_count,
        trend: 0,
        trend_label: "本周".to_string(),
        vitality: if employee_count > 0 { 85u8.min(60 + (employee_count / 10) as u8) } else { 0 },
        trend_percent: 0.0,
    };

    // 待处理和活动数据（暂无服务端 API，留空列表）
    let pending_items: Vec<PendingItem> = vec![];
    let activities: Vec<Activity> = vec![];

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
                    "Xilulu Console — 管理你的组织和团队 · {employee_count} 名员工 · {department_count} 个部门"
                }
            }

            // 主要内容网格
            div { style: "display: grid; grid-template-columns: 1fr 380px; gap: 24px;",

                // 左侧：统计卡片 + 活动时间线
                div { style: "display: flex; flex-direction: column; gap: 24px;",

                    // 统计卡片网格 — 用真实 API 数据
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
