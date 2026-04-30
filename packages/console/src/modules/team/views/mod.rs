//! ms-team 模块视图
//!
//! Phase 1: 全部为占位页面，显示页面标题 + "即将推出" 提示。

pub mod organizations;
pub mod departments;
pub mod employees;
pub mod positions;
pub mod contacts;

pub use organizations::TeamOrganizations;
pub use departments::TeamDepartments;
pub use employees::TeamEmployees;
pub use positions::TeamPositions;
pub use contacts::TeamContacts;
