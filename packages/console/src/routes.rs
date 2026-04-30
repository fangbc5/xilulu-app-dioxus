//! 路由定义
//!
//! 所有框架级和模块级路由在此静态定义。
//! 模块路由以 `/{module_id}/...` 为前缀。

use dioxus::prelude::*;
use crate::layout::ConsoleLayout;
use crate::views::cockpit::CockpitView;
use crate::modules::team::views::*;

/// 控制台路由定义
#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    // 所有页面共享 ConsoleLayout（顶栏 + 侧栏 + 内容区）
    #[layout(ConsoleLayout)]

        // === 框架级路由 ===
        #[route("/")]
        CockpitView {},

        // === ms-team 模块路由 ===
        #[route("/team/orgs")]
        TeamOrganizations {},

        #[route("/team/depts")]
        TeamDepartments {},

        #[route("/team/employees")]
        TeamEmployees {},

        #[route("/team/positions")]
        TeamPositions {},

        #[route("/team/contacts")]
        TeamContacts {},
}
