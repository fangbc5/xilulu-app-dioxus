//! 人员网格视图
//!
//! 员工卡片网格 + 搜索过滤。

use crate::views::people::detail_panel::use_detail_panel;
use crate::views::people::employee_card::{Employee, EmployeeCard};
use dioxus::prelude::*;

/// 视图模式
#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    Compact,
    Board,
}

#[component]
pub fn PeopleGrid(
    employees: Vec<Employee>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let view_mode = use_signal(|| ViewMode::Grid);

    // 过滤员工
    let q = search_query().to_lowercase();
    let filtered: Vec<Employee> = employees
        .iter()
        .filter(|e| {
            q.is_empty()
                || e.name.to_lowercase().contains(&q)
                || e.position.to_lowercase().contains(&q)
                || e.department.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px;",

            // 工具栏
            div { style: "display: flex; justify-content: space-between; align-items: center; gap: 16px;",

                // 搜索框
                div { style: "
                        flex: 1;
                        max-width: 320px;
                        position: relative;
                    ",
                    input {
                        style: "
                            width: 100%;
                            padding: 10px 14px 10px 40px;
                            background: var(--ds-bg-surface);
                            border: 1px solid var(--ds-border);
                            border-radius: var(--ds-radius-md);
                            color: var(--ds-text-primary);
                            font-size: 13.5px;
                            outline: none;
                            transition: all var(--ds-transition-fast);
                        ",
                        r#type: "text",
                        placeholder: "搜索姓名、职位、部门...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                    span { style: "
                            position: absolute;
                            left: 14px;
                            top: 50%;
                            transform: translateY(-50%);
                            color: var(--ds-text-tertiary);
                            font-size: 14px;
                        ",
                        "?"
                    }
                }

                // 视图切换
                div { style: "
                        display: flex;
                        gap: 4px;
                        background: var(--ds-bg-surface);
                        padding: 4px;
                        border-radius: var(--ds-radius-sm);
                    ",
                    ViewModeButton {
                        mode: ViewMode::Grid,
                        current: view_mode,
                        icon: "[]",
                    }
                    ViewModeButton {
                        mode: ViewMode::Compact,
                        current: view_mode,
                        icon: "=",
                    }
                    ViewModeButton {
                        mode: ViewMode::Board,
                        current: view_mode,
                        icon: "+",
                    }
                }

                // 结果数量
                div { style: "font-size: 13px; color: var(--ds-text-tertiary);",
                    "共 {filtered.len()} 人"
                }
            }

            // 卡片网格
            div { style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
                    gap: 16px;
                ",
                for emp in filtered.iter() {
                    EmployeeCardItem { employee: emp.clone() }
                }
            }

            // 空状态
            if filtered.is_empty() {
                div { class: "placeholder-view",
                    div { class: "placeholder-icon", "?" }
                    div { class: "placeholder-text", "没有找到匹配的员工" }
                }
            }
        }
    }
}

/// 员工卡片项目
#[component]
fn EmployeeCardItem(
    employee: Employee,
) -> Element {
    let detail_panel = use_detail_panel();

    rsx! {
        EmployeeCard {
            employee: employee.clone(),
            onclick: move |_| {
                detail_panel.open.call(employee.clone());
            }
        }
    }
}

/// 视图模式切换按钮
#[component]
fn ViewModeButton(
    mode: ViewMode,
    current: Signal<ViewMode>,
    icon: &'static str,
) -> Element {
    let is_active = current() == mode;
    let bg_color = if is_active { "var(--ds-accent-bg)" } else { "transparent" };
    let text_color = if is_active { "var(--ds-accent)" } else { "var(--ds-text-tertiary)" };

    rsx! {
        div {
            style: "
                width: 32px;
                height: 32px;
                display: flex;
                align-items: center;
                justify-content: center;
                border-radius: var(--ds-radius-sm);
                cursor: pointer;
                font-size: 14px;
                transition: all var(--ds-transition-fast);
                background: {bg_color};
                color: {text_color};
            ",
            onclick: move |_| current.set(mode),
            "{icon}"
        }
    }
}
