//! People 人员流视图
//!
//! 通讯录 + 员工管理，支持卡片网格、搜索、详情滑出面板。

pub mod employee_card;
pub mod detail_panel;
pub mod grid;

pub use employee_card::{Employee, EmployeeStatus};
pub use detail_panel::DetailPanelProvider;
pub use grid::PeopleGrid;

use crate::components::page_header::PageHeader;
use dioxus::prelude::*;

/// PeopleView 人员流视图
#[component]
pub fn PeopleView() -> Element {
    // 模拟员工数据
    let employees = get_mock_employees();

    rsx! {
        DetailPanelProvider {
            PageHeader {
                title: "人员管理".to_string(),
                description: Some("浏览员工、查看详情、管理部门分配".to_string()),
            }
            PeopleGrid { employees: employees }
        }
    }
}

/// 获取模拟员工数据
fn get_mock_employees() -> Vec<Employee> {
    vec![
        Employee {
            id: 1,
            name: "张伟".to_string(),
            avatar: None,
            position: "技术总监".to_string(),
            department: "技术中心".to_string(),
            phone: Some("138****1234".to_string()),
            email: Some("zhangwei@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2022.03".to_string(),
        },
        Employee {
            id: 2,
            name: "李娜".to_string(),
            avatar: None,
            position: "产品经理".to_string(),
            department: "产品部".to_string(),
            phone: Some("139****5678".to_string()),
            email: Some("lina@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2023.01".to_string(),
        },
        Employee {
            id: 3,
            name: "王芳".to_string(),
            avatar: None,
            position: "设计师".to_string(),
            department: "产品部".to_string(),
            phone: Some("137****9012".to_string()),
            email: Some("wangfang@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2023.06".to_string(),
        },
        Employee {
            id: 4,
            name: "刘强".to_string(),
            avatar: None,
            position: "高级工程师".to_string(),
            department: "AI实验室".to_string(),
            phone: Some("136****3456".to_string()),
            email: Some("liuqiang@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2022.09".to_string(),
        },
        Employee {
            id: 5,
            name: "陈静".to_string(),
            avatar: None,
            position: "运营专员".to_string(),
            department: "市场部".to_string(),
            phone: Some("135****7890".to_string()),
            email: Some("chenjing@xilulu.com".to_string()),
            status: EmployeeStatus::Onboarding,
            join_date: "2026.04".to_string(),
        },
        Employee {
            id: 6,
            name: "赵磊".to_string(),
            avatar: None,
            position: "前端工程师".to_string(),
            department: "技术中心".to_string(),
            phone: Some("134****2345".to_string()),
            email: Some("zhaolei@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2024.02".to_string(),
        },
        Employee {
            id: 7,
            name: "孙悦".to_string(),
            avatar: None,
            position: "财务主管".to_string(),
            department: "财务部".to_string(),
            phone: Some("133****6789".to_string()),
            email: Some("sunyue@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2021.11".to_string(),
        },
        Employee {
            id: 8,
            name: "周涛".to_string(),
            avatar: None,
            position: "市场总监".to_string(),
            department: "市场部".to_string(),
            phone: Some("132****0123".to_string()),
            email: Some("zhoutao@xilulu.com".to_string()),
            status: EmployeeStatus::Active,
            join_date: "2020.05".to_string(),
        },
    ]
}
