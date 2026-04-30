//! 模块注册中心
//!
//! 收集所有业务模块的元数据，提供给布局和命令面板使用。
//! 注册在启动时完成，之后只读取聚合数据。

use crate::module::{CockpitCard, Command, ConsoleModule, NavItem};

/// 模块基本信息
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
}

/// 已聚合的模块注册数据（可 Clone，适合注入 Context）
///
/// 在启动时从 `ConsoleModule` trait 对象提取所有数据，
/// 之后作为静态数据在整个应用中共享。
#[derive(Clone, Debug, PartialEq)]
pub struct RegistryData {
    /// 所有模块基本信息
    pub modules: Vec<ModuleInfo>,
    /// 按模块分组的导航项: (module_id, items)
    pub nav_groups: Vec<(String, Vec<NavItem>)>,
    /// 所有命令（命令面板用）
    pub commands: Vec<Command>,
    /// 所有驾驶舱卡片
    pub cockpit_cards: Vec<CockpitCard>,
}

impl RegistryData {
    /// 获取指定模块的导航项
    pub fn nav_items_for(&self, module_id: &str) -> Vec<NavItem> {
        self.nav_groups
            .iter()
            .find(|(id, _)| id == module_id)
            .map(|(_, items)| items.clone())
            .unwrap_or_default()
    }
}

/// 从模块列表构建注册数据
pub fn build_registry(modules: Vec<Box<dyn ConsoleModule>>) -> RegistryData {
    let module_infos: Vec<ModuleInfo> = modules
        .iter()
        .map(|m| ModuleInfo {
            id: m.id(),
            name: m.name(),
            icon: m.icon(),
            description: m.description(),
        })
        .collect();

    let nav_groups: Vec<(String, Vec<NavItem>)> = modules
        .iter()
        .map(|m| (m.id().to_string(), m.nav_items()))
        .collect();

    let commands: Vec<Command> = modules
        .iter()
        .flat_map(|m| m.commands())
        .collect();

    let cockpit_cards: Vec<CockpitCard> = modules
        .iter()
        .flat_map(|m| m.cockpit_cards())
        .collect();

    RegistryData {
        modules: module_infos,
        nav_groups,
        commands,
        cockpit_cards,
    }
}
