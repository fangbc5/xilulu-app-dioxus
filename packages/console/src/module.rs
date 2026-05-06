//! 控制台模块 trait 定义
//!
//! 每个微服务（ms-team、ms-oss、ms-content 等）实现 `ConsoleModule` trait，
//! 向框架注册自己的导航项、命令和驾驶舱卡片。

/// 控制台模块 trait —— 所有业务模块的统一接口
pub trait ConsoleModule {
    /// 模块唯一标识（如 "team"、"oss"、"content"）
    fn id(&self) -> &'static str;
    /// 模块显示名称（如 "组织管理"、"对象存储"）
    fn name(&self) -> &'static str;
    /// 模块图标（Lucide 图标名）
    fn icon(&self) -> &'static str;
    /// 模块描述
    fn description(&self) -> &'static str;
    /// 注册到侧边栏的导航项
    fn nav_items(&self) -> Vec<NavItem>;
    /// 注册到 ⌘K 命令面板的命令
    fn commands(&self) -> Vec<Command>;
    /// 驾驶舱首页展示的概览卡片
    fn cockpit_cards(&self) -> Vec<CockpitCard>;
}

/// 侧边栏导航项
#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    /// 显示文本
    pub label: String,
    /// 路由路径（如 "/team/employees"）
    pub path: String,
    /// Lucide 图标名
    pub icon: String,
    /// 角标数字（如未读数）
    pub badge: Option<u32>,
}

/// ⌘K 命令面板命令
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// 命令名称（搜索匹配用）
    pub name: String,
    /// 命令描述
    pub description: String,
    /// Lucide 图标名
    pub icon: String,
    /// 分组类别
    pub category: CommandCategory,
    /// 执行动作
    pub action: CommandAction,
}

/// 命令分类
#[derive(Clone, Debug, PartialEq)]
pub enum CommandCategory {
    /// 页面导航
    Navigation,
    /// 创建/编辑等操作
    Action,
    /// 搜索 (预留)
    #[allow(dead_code)]
    Search,
}

#[allow(dead_code)]
impl CommandCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Navigation => "导航",
            Self::Action => "操作",
            Self::Search => "搜索",
        }
    }
}

/// 命令执行动作
#[derive(Clone, Debug, PartialEq)]
pub enum CommandAction {
    /// 导航到指定路径
    Navigate(String),
    /// 打开侧面板（标识符）
    OpenPanel(String),
}

/// 驾驶舱概览卡片
#[derive(Clone, Debug, PartialEq)]
pub struct CockpitCard {
    /// 卡片标题
    pub title: String,
    /// 数值（如 "368"）
    pub value: String,
    /// 趋势文字（如 "△12 本周"）
    pub trend: Option<String>,
    /// 趋势是否正向
    pub trend_positive: bool,
    /// 关联的路由路径
    pub link: Option<String>,
}
