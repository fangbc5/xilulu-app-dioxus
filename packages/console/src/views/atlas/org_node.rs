//! 组织节点数据模型
//!
//! 组织树形结构的数据定义。

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
