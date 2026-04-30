use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

/// 岗位管理页面（占位）
#[component]
pub fn TeamPositions() -> Element {
    rsx! {
        PageHeader {
            title: "岗位管理".to_string(),
            description: Some("定义岗位体系、管理岗位层级和分类".to_string()),
        }
        div { class: "placeholder-view",
            div { class: "placeholder-icon", "💼" }
            div { class: "placeholder-text", "岗位管理视图即将推出" }
        }
    }
}
