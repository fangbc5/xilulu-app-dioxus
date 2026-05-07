//! 员工卡片组件
//!
//! 展示单个员工的信息卡片。

use crate::services::employee::EmployeeResponse;
use dioxus::prelude::*;

/// 员工状态
#[derive(Clone, Debug, PartialEq)]
pub enum EmployeeStatus {
    Active,
    /// 离职 (预留)
    #[allow(dead_code)]
    Inactive,
    Onboarding,
}

impl EmployeeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            EmployeeStatus::Active => "在职",
            EmployeeStatus::Inactive => "离职",
            EmployeeStatus::Onboarding => "入职中",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            EmployeeStatus::Active => "var(--ds-success)",
            EmployeeStatus::Inactive => "var(--ds-text-tertiary)",
            EmployeeStatus::Onboarding => "var(--ds-warning)",
        }
    }
}

/// 员工数据
#[derive(Clone, Debug, PartialEq)]
pub struct Employee {
    pub id: u64,
    pub name: String,
    pub avatar: Option<String>,
    pub position: String,
    pub department: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: EmployeeStatus,
    pub join_date: String,
}

impl Employee {
    /// 从 API 响应构建 Employee
    pub fn from_api(resp: EmployeeResponse) -> Self {
        let status = match resp.status {
            Some(1) => EmployeeStatus::Active,
            Some(0) => EmployeeStatus::Inactive,
            Some(2) => EmployeeStatus::Onboarding,
            _ => EmployeeStatus::Active,
        };

        Self {
            id: resp.id as u64,
            name: resp.name,
            avatar: resp.avatar,
            position: resp.primary_position_name.unwrap_or_else(|| "未分配职位".to_string()),
            department: resp.primary_department_name.unwrap_or_else(|| "未分配部门".to_string()),
            phone: resp.mobile,
            email: resp.email,
            status,
            join_date: resp.hire_date.unwrap_or_else(|| "—".to_string()),
        }
    }
}

/// 员工卡片组件
#[component]
pub fn EmployeeCard(
    employee: Employee,
    onclick: EventHandler<u64>,
) -> Element {
    let initials = employee
        .name
        .chars()
        .take(2)
        .collect::<String>()
        .to_uppercase();

    rsx! {
        div {
            class: "glass-card",
            style: "padding: 20px; cursor: pointer; transition: all var(--ds-transition-normal);",
            onclick: move |_| onclick.call(employee.id),

            // 头像 + 基本信息
            div { style: "display: flex; gap: 14px; margin-bottom: 16px;",

                // 头像
                div { style: "
                        width: 48px;
                        height: 48px;
                        border-radius: 50%;
                        background: linear-gradient(135deg, var(--ds-accent), var(--ds-accent-light));
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 16px;
                        font-weight: 600;
                        color: var(--ds-text-inverse);
                        flex-shrink: 0;
                    ",
                    if let Some(avatar_url) = &employee.avatar {
                        img {
                            src: "{avatar_url}",
                            alt: "{employee.name}",
                            style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover;",
                        }
                    } else {
                        "{initials}"
                    }
                }

                // 姓名 + 职位
                div { style: "flex: 1; min-width: 0;",
                    div { style: "font-size: 15px; font-weight: 600; color: var(--ds-text-primary); margin-bottom: 2px;",
                        "{employee.name}"
                    }
                    div { style: "font-size: 13px; color: var(--ds-text-secondary);",
                        "{employee.position}"
                    }
                    div { style: "font-size: 12px; color: var(--ds-text-tertiary); margin-top: 2px;",
                        "{employee.department}"
                    }
                }
            }

            // 分隔线
            div { style: "height: 1px; background: var(--ds-border); margin: 0 -20px 16px;" }

            // 联系信息
            div { style: "display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px;",
                if let Some(phone) = &employee.phone {
                    div { style: "display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: var(--ds-text-secondary);",
                        span { "+86" }
                        span { "{phone}" }
                    }
                }
                if let Some(email) = &employee.email {
                    div { style: "display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: var(--ds-text-secondary);",
                        span { "@" }
                        span { style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{email}"
                        }
                    }
                }
            }

            // 底部信息
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                // 入职时间
                div { style: "font-size: 11.5px; color: var(--ds-text-tertiary);",
                    "入职 {employee.join_date}"
                }

                // 状态标签
                div { style: "
                        padding: 3px 10px;
                        border-radius: 12px;
                        font-size: 11px;
                        font-weight: 500;
                        background: var(--ds-bg-surface);
                        color: {employee.status.color()};
                    ",
                    "{employee.status.label()}"
                }
            }
        }
    }
}
