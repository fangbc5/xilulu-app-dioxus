//! 待处理事项 (Pending Items)
//!
//! 显示需要处理的任务列表。

use dioxus::prelude::*;

/// 待处理事项类型
#[derive(Clone, Debug, PartialEq)]
pub enum PendingType {
    /// 新员工待分配部门
    NewEmployee,
    /// 部门申请
    DepartmentRequest,
    /// 调岗申请
    TransferRequest,
    /// 审批提醒 (预留)
    #[allow(dead_code)]
    Approval,
}

impl PendingType {
    pub fn icon(&self) -> &'static str {
        match self {
            PendingType::NewEmployee => "+",
            PendingType::DepartmentRequest => "?",
            PendingType::TransferRequest => "~",
            PendingType::Approval => "!",
        }
    }
}

/// 待处理事项
#[derive(Clone, Debug, PartialEq)]
pub struct PendingItem {
    pub id: u64,
    pub r#type: PendingType,
    pub title: String,
    pub description: String,
    pub time: String,
}

/// 待处理卡片列表组件
#[component]
pub fn PendingList(
    items: Vec<PendingItem>,
    on_item_click: Callback<u64>,
) -> Element {
    rsx! {
        div {
            style: "padding: 16px; display: flex; flex-direction: column; gap: 8px;",

            // 标题
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;",
                span {
                    style: "font-size: 13px; font-weight: 600; color: var(--ds-text-primary);",
                    "待处理"
                }
                if !items.is_empty() {
                    span {
                        style: "font-size: 12px; padding: 2px 8px; background: var(--ds-accent-bg); color: var(--ds-accent); border-radius: 10px; font-weight: 500;",
                        "{items.len()} 项"
                    }
                }
            }

            // 列表
            if items.is_empty() {
                div {
                    style: "padding: 24px; text-align: center; color: var(--ds-text-tertiary); font-size: 13px;",
                    "暂无待处理事项"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 6px;",
                    for item in items.iter() {
                        PendingItemRow {
                            item: item.clone(),
                            on_select: on_item_click.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// 单个待处理事项行
#[component]
fn PendingItemRow(
    item: PendingItem,
    on_select: Callback<u64>,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                gap: 12px;
                padding: 10px 12px;
                background: var(--ds-bg-surface);
                border-radius: var(--ds-radius-md);
                cursor: pointer;
                transition: all var(--ds-transition-fast);
                border: 1px solid transparent;
            ",
            onclick: {
                let id = item.id;
                move |_| { on_select.call(id); }
            },

            // 图标
            div {
                style: "
                    width: 32px;
                    height: 32px;
                    border-radius: var(--ds-radius-sm);
                    background: var(--ds-bg-elevated);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    flex-shrink: 0;
                ",
                "{item.r#type.icon()}"
            }

            // 内容
            div {
                style: "flex: 1; min-width: 0;",
                div {
                    style: "font-size: 13px; font-weight: 500; color: var(--ds-text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{item.title}"
                }
                div {
                    style: "font-size: 11.5px; color: var(--ds-text-tertiary); margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{item.description}"
                }
            }

            // 时间
            div {
                style: "font-size: 11px; color: var(--ds-text-tertiary); flex-shrink: 0;",
                "{item.time}"
            }

            // 箭头
            div {
                style: "color: var(--ds-text-tertiary); font-size: 14px;",
                ">"
            }
        }
    }
}
