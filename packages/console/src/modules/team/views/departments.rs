use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

/// 部门管理页面（占位）
#[component]
pub fn TeamDepartments() -> Element {
    rsx! {
        PageHeader {
            title: "部门管理".to_string(),
            description: Some("管理部门树结构、设置负责人、查看人员统计".to_string()),
        }
        div { class: "placeholder-view",
            div { class: "placeholder-icon", "📂" }
            div { class: "placeholder-text", "部门管理视图即将推出" }
        }
    }
}
