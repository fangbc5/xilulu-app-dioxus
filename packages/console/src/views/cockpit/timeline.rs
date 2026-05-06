//! 最近操作时间线 (Recent Activity Timeline)
//!
//! 显示最近的系统操作记录。

use dioxus::prelude::*;

/// 操作类型
#[derive(Clone, Debug, PartialEq)]
pub enum ActivityType {
    Create,
    Update,
    /// 删除 (预留)
    #[allow(dead_code)]
    Delete,
    View,
    Transfer,
}

impl ActivityType {
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityType::Create => "+",
            ActivityType::Update => "~",
            ActivityType::Delete => "-",
            ActivityType::View => "*",
            ActivityType::Transfer => "~",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActivityType::Create => "创建",
            ActivityType::Update => "更新",
            ActivityType::Delete => "删除",
            ActivityType::View => "查看",
            ActivityType::Transfer => "调岗",
        }
    }
}

/// 操作记录
#[derive(Clone, Debug, PartialEq)]
pub struct Activity {
    pub id: u64,
    pub r#type: ActivityType,
    pub actor: String,
    pub target: String,
    pub target_type: String,
    pub time: String,
    pub time_ago: String,
}

/// 最近活动时间线组件
#[component]
pub fn ActivityTimeline(
    activities: Vec<Activity>,
) -> Element {
    rsx! {
        div {
            style: "padding: 16px; display: flex; flex-direction: column; gap: 0;",

            // 标题
            div {
                style: "font-size: 13px; font-weight: 600; color: var(--ds-text-primary); margin-bottom: 16px;",
                "最近活动"
            }

            // 时间线
            if activities.is_empty() {
                div {
                    style: "padding: 24px; text-align: center; color: var(--ds-text-tertiary); font-size: 13px;",
                    "暂无活动记录"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column;",
                    for (i, activity) in activities.iter().enumerate() {
                        ActivityItem {
                            activity: activity.clone(),
                            is_last: i == activities.len() - 1,
                        }
                    }
                }
            }
        }
    }
}

/// 单个活动记录
#[component]
fn ActivityItem(
    activity: Activity,
    is_last: bool,
) -> Element {
    rsx! {
        div {
            style: "display: flex; gap: 12px; position: relative;",

            // 时间线连接线
            if !is_last {
                div {
                    style: "
                        position: absolute;
                        left: 15px;
                        top: 32px;
                        bottom: -12px;
                        width: 1px;
                        background: var(--ds-border);
                    "
                }
            }

            // 图标
            div {
                style: "
                    width: 30px;
                    height: 30px;
                    border-radius: 50%;
                    background: var(--ds-bg-surface);
                    border: 1px solid var(--ds-border);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 12px;
                    flex-shrink: 0;
                    z-index: 1;
                ",
                "{activity.r#type.icon()}"
            }

            // 内容
            div {
                style: "flex: 1; padding-bottom: 16px;",
                div {
                    style: "font-size: 13px; color: var(--ds-text-primary); line-height: 1.4;",
                    span {
                        style: "color: var(--ds-text-secondary);",
                        "{activity.actor}"
                    }
                    " "
                    "{activity.r#type.label()}"
                    "了 "
                    span {
                        style: "color: var(--ds-accent); font-weight: 500;",
                        "{activity.target}"
                    }
                }
                div {
                    style: "display: flex; justify-content: space-between; margin-top: 4px;",
                    span {
                        style: "font-size: 11px; color: var(--ds-text-tertiary);",
                        "{activity.target_type}"
                    }
                    span {
                        style: "font-size: 11px; color: var(--ds-text-tertiary);",
                        "{activity.time_ago}"
                    }
                }
            }
        }
    }
}
