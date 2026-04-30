//! ms-team 模块 —— 组织与人员管理
//!
//! 第一个接入 Console 的业务模块。
//! 注册组织、部门、员工、岗位、通讯录相关的导航和命令。

pub mod views;

use crate::module::*;

/// ms-team 模块实现
pub struct TeamModule;

impl ConsoleModule for TeamModule {
    fn id(&self) -> &'static str {
        "team"
    }

    fn name(&self) -> &'static str {
        "组织管理"
    }

    fn icon(&self) -> &'static str {
        "building-2"
    }

    fn description(&self) -> &'static str {
        "组织架构、部门管理、员工管理、岗位体系"
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![
            NavItem {
                label: "组织".to_string(),
                path: "/team/orgs".to_string(),
                icon: "🏢".to_string(),
                badge: None,
            },
            NavItem {
                label: "部门".to_string(),
                path: "/team/depts".to_string(),
                icon: "📂".to_string(),
                badge: None,
            },
            NavItem {
                label: "员工".to_string(),
                path: "/team/employees".to_string(),
                icon: "👤".to_string(),
                badge: Some(3), // 假数据：3 位待分配员工
            },
            NavItem {
                label: "岗位".to_string(),
                path: "/team/positions".to_string(),
                icon: "💼".to_string(),
                badge: None,
            },
            NavItem {
                label: "通讯录".to_string(),
                path: "/team/contacts".to_string(),
                icon: "📖".to_string(),
                badge: None,
            },
        ]
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            // 导航命令
            Command {
                name: "打开组织管理".to_string(),
                description: "查看组织列表".to_string(),
                icon: "building".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate("/team/orgs".to_string()),
            },
            Command {
                name: "打开部门管理".to_string(),
                description: "查看部门树".to_string(),
                icon: "folder-tree".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate("/team/depts".to_string()),
            },
            Command {
                name: "打开员工管理".to_string(),
                description: "浏览员工列表".to_string(),
                icon: "users".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate("/team/employees".to_string()),
            },
            Command {
                name: "打开岗位管理".to_string(),
                description: "管理岗位体系".to_string(),
                icon: "briefcase".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate("/team/positions".to_string()),
            },
            Command {
                name: "打开通讯录".to_string(),
                description: "搜索和浏览联系人".to_string(),
                icon: "book-user".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate("/team/contacts".to_string()),
            },
            // 操作命令
            Command {
                name: "创建员工".to_string(),
                description: "新增一名员工".to_string(),
                icon: "user-plus".to_string(),
                category: CommandCategory::Action,
                action: CommandAction::OpenPanel("create-employee".to_string()),
            },
            Command {
                name: "创建部门".to_string(),
                description: "新增一个部门".to_string(),
                icon: "folder-plus".to_string(),
                category: CommandCategory::Action,
                action: CommandAction::OpenPanel("create-department".to_string()),
            },
        ]
    }

    fn cockpit_cards(&self) -> Vec<CockpitCard> {
        // Phase 1: 假数据
        vec![
            CockpitCard {
                title: "在职员工".to_string(),
                value: "368".to_string(),
                trend: Some("△12 本周".to_string()),
                trend_positive: true,
                link: Some("/team/employees".to_string()),
            },
            CockpitCard {
                title: "部门数量".to_string(),
                value: "24".to_string(),
                trend: Some("△1 本月".to_string()),
                trend_positive: true,
                link: Some("/team/depts".to_string()),
            },
            CockpitCard {
                title: "岗位数量".to_string(),
                value: "42".to_string(),
                trend: None,
                trend_positive: true,
                link: Some("/team/positions".to_string()),
            },
            CockpitCard {
                title: "本月入职".to_string(),
                value: "8".to_string(),
                trend: Some("▽3 vs 上月".to_string()),
                trend_positive: false,
                link: None,
            },
        ]
    }
}
