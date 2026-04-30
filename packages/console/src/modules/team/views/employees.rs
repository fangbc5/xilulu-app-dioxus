use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

/// 员工管理页面（占位）
#[component]
pub fn TeamEmployees() -> Element {
    rsx! {
        PageHeader {
            title: "员工管理".to_string(),
            description: Some("浏览员工列表、管理入职离职、调整部门岗位".to_string()),
        }
        div { class: "placeholder-view",
            div { class: "placeholder-icon", "👤" }
            div { class: "placeholder-text", "员工管理视图即将推出" }
        }
    }
}
