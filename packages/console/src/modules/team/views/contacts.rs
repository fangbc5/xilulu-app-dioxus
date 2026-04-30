use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

/// 通讯录页面（占位）
#[component]
pub fn TeamContacts() -> Element {
    rsx! {
        PageHeader {
            title: "通讯录".to_string(),
            description: Some("浏览和搜索企业通讯录、查看联系人详情".to_string()),
        }
        div { class: "placeholder-view",
            div { class: "placeholder-icon", "📖" }
            div { class: "placeholder-text", "通讯录视图即将推出" }
        }
    }
}
