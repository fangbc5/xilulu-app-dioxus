//! 组织节点数据模型
//!
//! 组织树形结构的数据定义。

use crate::services::department::DepartmentTreeNode;
use serde::{Deserialize, Serialize};

pub const HISTORY_MONTHS: [&str; 6] = [
    "2025.12",
    "2026.01",
    "2026.02",
    "2026.03",
    "2026.04",
    "2026.05",
];

/// 组织节点
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
    /// 节点 ID
    pub id: u64,
    /// 节点名称
    pub name: String,
    /// 节点类型
    pub node_type: OrgNodeType,
    /// 父节点 ID（None 表示根节点）
    pub parent_id: Option<u64>,
    /// 子节点
    pub children: Vec<OrgNode>,
    /// 成员数量
    pub member_count: u32,
    /// 是否展开
    pub expanded: bool,
    /// 最近 6 个月的规模变化
    pub history: Vec<u32>,
    /// 人员增长率
    pub growth_rate: f32,
    /// 协作热度指数
    pub collaboration_index: f32,
}

/// 节点类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrgNodeType {
    /// 公司/集团
    Company,
    /// 部门
    Department,
    /// 小组
    Group,
}

impl OrgNode {
    /// 创建根节点
    pub fn root(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            node_type: OrgNodeType::Company,
            parent_id: None,
            children: Vec::new(),
            member_count: 0,
            expanded: true,
            history: Vec::new(),
            growth_rate: 0.0,
            collaboration_index: 0.0,
        }
    }

    /// 创建叶子节点
    pub fn leaf(
        id: u64,
        name: &str,
        node_type: OrgNodeType,
        parent_id: Option<u64>,
        member_count: u32,
        history: Vec<u32>,
        collaboration_index: f32,
    ) -> Self {
        let growth_rate = growth_rate_from_history(&history);
        Self {
            id,
            name: name.to_string(),
            node_type,
            parent_id,
            children: Vec::new(),
            member_count,
            expanded: false,
            history,
            growth_rate,
            collaboration_index,
        }
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: OrgNode) {
        self.children.push(child);
    }

    /// 重新计算汇总节点的规模和趋势
    pub fn refresh_rollup_metrics(&mut self) {
        for child in &mut self.children {
            child.refresh_rollup_metrics();
        }

        if self.children.is_empty() {
            if self.history.is_empty() {
                self.history = vec![self.member_count];
            }
            self.growth_rate = growth_rate_from_history(&self.history);
            return;
        }

        self.member_count = self.children.iter().map(|child| child.member_count).sum();

        let timeline_len = self
            .children
            .iter()
            .map(OrgNode::timeline_len)
            .max()
            .unwrap_or(1);

        self.history = (0..timeline_len)
            .map(|index| self.children.iter().map(|child| child.headcount_at(index)).sum())
            .collect();

        let average_heat = self
            .children
            .iter()
            .map(|child| child.collaboration_index)
            .sum::<f32>()
            / self.children.len() as f32;
        let branching_bonus = (self.children.len() as f32 * 2.5).min(10.0);
        self.collaboration_index = (average_heat + branching_bonus).min(100.0);
        self.growth_rate = growth_rate_from_history(&self.history);
    }

    /// 根据节点 ID 查找节点
    pub fn find(&self, node_id: u64) -> Option<&OrgNode> {
        if self.id == node_id {
            return Some(self);
        }

        self.children
            .iter()
            .find_map(|child| child.find(node_id))
    }

    /// 切换展开状态
    pub fn toggle_expanded(&mut self, node_id: u64) -> bool {
        if self.id == node_id {
            if !self.children.is_empty() {
                self.expanded = !self.expanded;
            }
            return true;
        }

        self.children
            .iter_mut()
            .any(|child| child.toggle_expanded(node_id))
    }

    /// 获取当前可见节点数
    pub fn visible_node_count(&self) -> usize {
        if !self.expanded {
            return 1;
        }

        1 + self
            .children
            .iter()
            .map(OrgNode::visible_node_count)
            .sum::<usize>()
    }

    /// 获取时间轴长度
    pub fn timeline_len(&self) -> usize {
        self.history.len().max(1)
    }

    /// 获取某个时间点的人数
    pub fn headcount_at(&self, index: usize) -> u32 {
        self.history
            .get(index)
            .copied()
            .or_else(|| self.history.last().copied())
            .unwrap_or(self.member_count)
    }

    /// 计算节点半径
    pub fn radius_for(node_type: OrgNodeType, member_count: u32) -> f32 {
        let base = match node_type {
            OrgNodeType::Company => 42.0,
            OrgNodeType::Department => 32.0,
            OrgNodeType::Group => 24.0,
        };
        base + (member_count as f32 * 0.25).min(18.0)
    }

    /// 从 API 部门树节点构建 OrgNode 树
    pub fn from_api_tree(nodes: Vec<DepartmentTreeNode>, parent_id: Option<u64>) -> Vec<OrgNode> {
        nodes
            .into_iter()
            .map(|node| {
                let member_count = node.employee_count.unwrap_or(0) as u32;
                let has_children = node.children
                    .as_ref()
                    .map(|c| !c.is_empty())
                    .unwrap_or(false);

                let children = node.children
                    .unwrap_or_default();
                let child_nodes = Self::from_api_tree(children, Some(node.id as u64));

                let node_type = if parent_id.is_none() && has_children {
                    OrgNodeType::Company
                } else if has_children {
                    OrgNodeType::Department
                } else {
                    OrgNodeType::Group
                };

                let expanded = has_children;

                let mut org_node = OrgNode {
                    id: node.id as u64,
                    name: node.name,
                    node_type,
                    parent_id,
                    children: child_nodes,
                    member_count,
                    expanded,
                    history: vec![member_count],
                    growth_rate: 0.0,
                    collaboration_index: 50.0,
                };
                org_node.refresh_rollup_metrics();
                org_node
            })
            .collect()
    }

}

/// 力导向图节点（运行时状态）
#[derive(Clone, Debug, PartialEq)]
pub struct ForceNode {
    pub id: u64,
    pub name: String,
    pub node_type: OrgNodeType,
    pub member_count: u32,
    pub history: Vec<u32>,
    pub growth_rate: f32,
    pub collaboration_index: f32,
    pub depth: usize,
    pub expanded: bool,
    pub child_count: usize,
    pub home_x: f32,
    pub home_y: f32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub fx: Option<f32>,  // 固定 x 位置
    pub fy: Option<f32>,  // 固定 y 位置
    pub radius: f32,
}

impl ForceNode {
    pub fn from_org(node: &OrgNode, depth: usize) -> Self {
        Self {
            id: node.id,
            name: node.name.clone(),
            node_type: node.node_type,
            member_count: node.member_count,
            history: node.history.clone(),
            growth_rate: node.growth_rate,
            collaboration_index: node.collaboration_index,
            depth,
            expanded: node.expanded,
            child_count: node.children.len(),
            home_x: 0.0,
            home_y: 0.0,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            fx: None,
            fy: None,
            radius: OrgNode::radius_for(node.node_type, node.member_count),
        }
    }

    pub fn headcount_at(&self, index: usize) -> u32 {
        self.history
            .get(index)
            .copied()
            .or_else(|| self.history.last().copied())
            .unwrap_or(self.member_count)
    }

    pub fn radius_at(&self, index: usize) -> f32 {
        OrgNode::radius_for(self.node_type, self.headcount_at(index))
    }
}

/// 力导向图边
#[derive(Clone, Debug, PartialEq)]
pub struct ForceLink {
    pub source: u64,
    pub target: u64,
    pub distance: f32,
    pub weight: f32,
}

/// 拖拽变更预览数据
#[derive(Clone, Debug, PartialEq)]
pub struct DragChangePreview {
    /// 被拖拽的节点 ID
    pub node_id: u64,
    /// 被拖拽的节点名称
    pub node_name: String,
    /// 原父节点 ID
    pub old_parent_id: u64,
    /// 原父节点名称
    pub old_parent_name: String,
    /// 新父节点 ID
    pub new_parent_id: u64,
    /// 新父节点名称
    pub new_parent_name: String,
    /// 被拖拽节点的人数
    pub member_count: u32,
    /// 原父节点下剩余子节点数
    pub old_parent_remaining_children: usize,
    /// 新父节点原有子节点数
    pub new_parent_existing_children: usize,
}

impl OrgNode {
    /// 将指定节点从当前父节点移动到新父节点
    ///
    /// 返回 true 表示移动成功，false 表示无法移动（如目标是自身后代）
    pub fn move_to_parent(&mut self, node_id: u64, new_parent_id: u64) -> bool {
        // 不能移到自己下面
        if node_id == new_parent_id {
            return false;
        }

        // 检查目标节点是否是被拖拽节点的后代
        if let Some(dragged) = self.find(node_id) {
            if dragged.find(new_parent_id).is_some() {
                return false; // 不能移到自己的后代下面
            }
        }

        // 从原父节点中移除
        let removed = self.remove_child(node_id);
        let Some(mut removed_node) = removed else {
            return false;
        };

        // 更新 parent_id
        removed_node.parent_id = Some(new_parent_id);

        // 添加到新父节点
        if self.add_child_to(new_parent_id, removed_node) {
            // 重新计算汇总指标
            self.refresh_rollup_metrics();
            true
        } else {
            false
        }
    }

    /// 生成拖拽变更预览数据
    pub fn preview_move(&self, node_id: u64, new_parent_id: u64) -> Option<DragChangePreview> {
        if node_id == new_parent_id {
            return None;
        }

        let dragged = self.find(node_id)?;
        
        // 检查目标是否是后代
        if dragged.find(new_parent_id).is_some() {
            return None;
        }

        let old_parent_id = dragged.parent_id?;
        let old_parent = self.find(old_parent_id)?;
        let new_parent = self.find(new_parent_id)?;

        Some(DragChangePreview {
            node_id,
            node_name: dragged.name.clone(),
            old_parent_id,
            old_parent_name: old_parent.name.clone(),
            new_parent_id,
            new_parent_name: new_parent.name.clone(),
            member_count: dragged.member_count,
            old_parent_remaining_children: old_parent.children.len().saturating_sub(1),
            new_parent_existing_children: new_parent.children.len(),
        })
    }

    /// 从子节点中移除指定 ID 的节点（深度优先搜索）
    fn remove_child(&mut self, node_id: u64) -> Option<OrgNode> {
        for i in 0..self.children.len() {
            if self.children[i].id == node_id {
                return Some(self.children.remove(i));
            }
        }

        for child in &mut self.children {
            if let Some(removed) = child.remove_child(node_id) {
                return Some(removed);
            }
        }

        None
    }

    /// 向指定 ID 的节点添加子节点
    fn add_child_to(&mut self, parent_id: u64, child: OrgNode) -> bool {
        if self.id == parent_id {
            self.children.push(child);
            return true;
        }

        // 先找到目标所在的子树，再传递所有权
        for c in &mut self.children {
            if c.find(parent_id).is_some() {
                return c.add_child_to(parent_id, child);
            }
        }

        false
    }

}

fn growth_rate_from_history(history: &[u32]) -> f32 {
    if history.len() < 2 {
        return 0.0;
    }

    let previous = history[history.len() - 2] as f32;
    let current = history[history.len() - 1] as f32;

    if previous <= f32::EPSILON {
        return 0.0;
    }

    ((current - previous) / previous) * 100.0
}
