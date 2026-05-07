//! 路由定义
//!
//! 所有框架级和模块级路由在此静态定义。
//! 模块路由以 `/{module_id}/...` 为前缀。

use dioxus::prelude::*;
use crate::layout::ConsoleLayout;
use crate::views::atlas::AtlasView;
use crate::views::auth::{LoginPage, SelectTenantPage};
use crate::views::cockpit::CockpitView;
use crate::views::errors::NotFoundPage;
use crate::views::people::PeopleView;
use crate::modules::team::views::*;

/// 控制台路由定义
#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    // === 认证路由（无 Layout） ===
    #[route("/login")]
    LoginPage {},

    #[route("/select-tenant")]
    SelectTenantPage {},

    // 所有业务页面共享 ConsoleLayout（顶栏 + 侧栏 + 内容区）
    #[layout(ConsoleLayout)]

        // === 框架级路由 ===
        #[route("/")]
        CockpitView {},

        #[route("/people")]
        PeopleView {},

        #[route("/atlas")]
        AtlasView {},

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

    // === 404 兜底 ===
    #[route("/:..route")]
    NotFoundPage { route: Vec<String> },
}