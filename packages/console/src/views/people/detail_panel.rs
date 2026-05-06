//! 员工详情滑出面板
//!
//! 从右侧滑出的员工详情面板，支持内联编辑。

use crate::views::people::employee_card::Employee;
use dioxus::prelude::*;

/// 详情面板 Context
#[derive(Clone, Copy)]
pub struct DetailPanelContext {
    pub open: Callback<Employee>,
    pub close: Callback<()>,
}

pub fn use_detail_panel() -> DetailPanelContext {
    let ctx = use_context::<DetailPanelContext>();
    ctx
}

/// 详情面板 Provider
#[component]
pub fn DetailPanelProvider(
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| false);
    let mut selected_employee = use_signal(|| None);

    let ctx = DetailPanelContext {
        open: Callback::new(move |emp| {
            selected_employee.set(Some(emp));
            is_open.set(true);
        }),
        close: Callback::new(move |_| {
            is_open.set(false);
        }),
    };

    use_context_provider(|| ctx);

    rsx! {
        {children}
        if is_open() {
            DetailPanelOverlay {
                employee: selected_employee(),
                close_fn: ctx.close,
            }
        }
    }
}

/// 详情面板浮层
#[component]
fn DetailPanelOverlay(
    employee: Option<Employee>,
    close_fn: Callback<()>,
) -> Element {
    rsx! {
        // 半透明背景
        div {
            style: "
                position: fixed;
                inset: 0;
                background: var(--ds-overlay-bg);
                z-index: 150;
                animation: overlay-appear 150ms ease;
            ",
            onclick: {
                let close_fn = close_fn;
                move |_| { close_fn.call(()); }
            },
        }

        // 滑出面板
        div {
            style: "
                position: fixed;
                top: 0;
                right: 0;
                bottom: 0;
                width: 420px;
                max-width: 90vw;
                background: var(--ds-bg-elevated);
                border-left: 1px solid var(--ds-border);
                z-index: 160;
                display: flex;
                flex-direction: column;
                animation: panel-slide-in 250ms cubic-bezier(0.16, 1, 0.3, 1);
                overflow: hidden;
            ",
            onclick: move |e| e.stop_propagation(),

            // 头部
            DetailPanelHeader {
                close_fn: close_fn,
            }

            // 内容区
            if let Some(emp) = employee {
                EmployeeDetailContent { employee: emp }
            }
        }
    }
}

/// 详情面板头部
#[component]
fn DetailPanelHeader(
    close_fn: Callback<()>,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 16px 20px;
                border-bottom: 1px solid var(--ds-border);
                background: var(--ds-bg-surface);
            ",
            // 关闭按钮
            div {
                style: "
                    width: 32px;
                    height: 32px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    border-radius: var(--ds-radius-sm);
                    cursor: pointer;
                    color: var(--ds-text-secondary);
                    font-size: 18px;
                    transition: all var(--ds-transition-fast);
                ",
                onclick: {
                    let close_fn = close_fn;
                    move |_| { close_fn.call(()); }
                },
                "x"
            }

            // 动作按钮
            div {
                style: "display: flex; gap: 8px;",
                button {
                    style: "
                        padding: 6px 14px;
                        border-radius: var(--ds-radius-sm);
                        font-size: 13px;
                        cursor: pointer;
                        transition: all var(--ds-transition-fast);
                        background: var(--ds-accent-bg);
                        color: var(--ds-accent);
                        border: 1px solid var(--ds-border-accent);
                    ",
                    "编辑"
                }
                button {
                    style: "
                        padding: 6px 14px;
                        border-radius: var(--ds-radius-sm);
                        font-size: 13px;
                        cursor: pointer;
                        transition: all var(--ds-transition-fast);
                        background: var(--ds-accent);
                        color: var(--ds-text-inverse);
                        border: none;
                    ",
                    "发消息"
                }
            }
        }
    }
}

/// 员工详情内容
#[component]
fn EmployeeDetailContent(
    employee: Employee,
) -> Element {
    let initials = employee.name.chars().take(2).collect::<String>().to_uppercase();

    rsx! {
        div {
            style: "flex: 1; overflow-y: auto; padding: 24px;",

            // 头像 + 姓名
            div {
                style: "display: flex; flex-direction: column; align-items: center; text-align: center; margin-bottom: 28px;",

                div {
                    style: "
                        width: 80px;
                        height: 80px;
                        border-radius: 50%;
                        background: linear-gradient(135deg, var(--ds-accent), var(--ds-accent-light));
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 28px;
                        font-weight: 600;
                        color: var(--ds-text-inverse);
                        margin-bottom: 16px;
                        box-shadow: 0 8px 32px rgba(124, 58, 237, 0.3);
                    ",
                    "{initials}"
                }

                h2 {
                    style: "font-size: 22px; font-weight: 600; color: var(--ds-text-primary); margin-bottom: 4px;",
                    "{employee.name}"
                }

                div {
                    style: "font-size: 14px; color: var(--ds-text-secondary); margin-bottom: 8px;",
                    "{employee.position}"
                }

                div {
                    style: "
                        display: inline-flex;
                        padding: 4px 12px;
                        border-radius: 12px;
                        font-size: 12px;
                        font-weight: 500;
                        background: var(--ds-bg-surface);
                        color: {employee.status.color()};
                    ",
                    "{employee.status.label()}"
                }
            }

            // 信息卡片
            div {
                style: "display: flex; flex-direction: column; gap: 16px;",

                // 基本信息
                InfoSection {
                    title: "基本信息".to_string(),
                    items: vec![
                        ("部门".to_string(), employee.department.clone()),
                        ("入职日期".to_string(), employee.join_date.clone()),
                        ("手机".to_string(), employee.phone.clone().unwrap_or_default()),
                        ("邮箱".to_string(), employee.email.clone().unwrap_or_default()),
                    ],
                }
            }
        }
    }
}

/// 信息区域组件
#[component]
fn InfoSection(
    title: String,
    items: Vec<(String, String)>,
) -> Element {
    rsx! {
        div {
            style: "
                background: var(--ds-bg-surface);
                border-radius: var(--ds-radius-md);
                padding: 16px;
            ",
            div {
                style: "font-size: 12px; font-weight: 600; color: var(--ds-text-tertiary); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px;",
                "{title}"
            }
            div {
                style: "display: flex; flex-direction: column; gap: 10px;",
                for (label, value) in items.iter() {
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        span {
                            style: "font-size: 13px; color: var(--ds-text-secondary);",
                            "{label}"
                        }
                        span {
                            style: "font-size: 13px; color: var(--ds-text-primary);",
                            "{value}"
                        }
                    }
                }
            }
        }
    }
}
