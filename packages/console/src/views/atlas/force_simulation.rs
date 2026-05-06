//! 力导向图引擎
//!
//! 实现 D3-style 的力导向布局算法。

use super::org_node::{ForceLink, ForceNode};
use std::collections::BTreeMap;

/// 力导向模拟器
pub struct ForceSimulation {
    /// 节点列表
    nodes: Vec<ForceNode>,
    /// 边列表
    links: Vec<ForceLink>,
    /// 参数配置
    pub alpha: f32,
    pub alpha_decay: f32,
    pub velocity_decay: f32,
    pub link_strength: f32,
    pub charge_strength: f32,
    pub center_strength: f32,
    pub collision_padding: f32,
    pub width: f32,
    pub height: f32,
    /// 迭代计数
    pub iterations: usize,
}

impl ForceSimulation {
    /// 创建新的模拟器
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            alpha: 1.0,
            alpha_decay: 0.02,
            velocity_decay: 0.4,
            link_strength: 0.22,
            charge_strength: -4200.0,
            center_strength: 0.16,
            collision_padding: 30.0,
            width,
            height,
            iterations: 0,
        }
    }

    /// 设置节点
    pub fn set_nodes(&mut self, nodes: Vec<ForceNode>) {
        self.nodes = nodes;
        self.seed_by_depth();
    }

    /// 设置边
    pub fn set_links(&mut self, links: Vec<ForceLink>) {
        self.links = links;
    }

    /// 获取节点引用
    pub fn nodes(&self) -> &[ForceNode] {
        &self.nodes
    }

    /// 获取边引用
    pub fn links(&self) -> &[ForceLink] {
        &self.links
    }

    /// 运行一次迭代
    pub fn tick(&mut self) {
        if self.alpha < 0.001 {
            return;
        }

        self.iterations += 1;

        // 应用力
        self.apply_link_force();
        self.apply_charge_force();
        self.apply_collision_force();
        self.apply_center_force();

        // 更新位置
        self.apply_velocity();

        // 衰减 alpha
        self.alpha = (self.alpha - self.alpha_decay).max(0.0);
    }

    /// 运行直到收敛
    pub fn run(&mut self, max_iterations: usize) {
        for _ in 0..max_iterations {
            self.tick();
            if self.alpha < 0.001 {
                break;
            }
        }
    }

    /// 链接力（弹簧力）
    fn apply_link_force(&mut self) {
        for link in &self.links {
            let Some(source_idx) = self.nodes.iter().position(|n| n.id == link.source) else {
                continue;
            };
            let Some(target_idx) = self.nodes.iter().position(|n| n.id == link.target) else {
                continue;
            };

            let (source, target) = if source_idx < target_idx {
                let (left, right) = self.nodes.split_at_mut(target_idx);
                (&mut left[source_idx], &mut right[0])
            } else if source_idx > target_idx {
                let (left, right) = self.nodes.split_at_mut(source_idx);
                (&mut right[0], &mut left[target_idx])
            } else {
                continue;
            };

            let dx = target.x - source.x;
            let dy = target.y - source.y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);

            // 胡克定律：F = k * (x - x0)
            let force =
                self.link_strength * link.weight * self.alpha * (distance - link.distance);

            let fx = force * dx / distance;
            let fy = force * dy / distance;

            source.vx += fx;
            source.vy += fy;
            target.vx -= fx;
            target.vy -= fy;
        }
    }

    /// 电荷力（排斥力）
    fn apply_charge_force(&mut self) {
        let n = self.nodes.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.nodes[j].x - self.nodes[i].x;
                let dy = self.nodes[j].y - self.nodes[i].y;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                // 库仑定律：F = k * q1 * q2 / r^2
                let force = self.charge_strength * self.alpha / (distance * distance);

                let fx = force * dx / distance;
                let fy = force * dy / distance;

                self.nodes[i].vx += fx;
                self.nodes[i].vy += fy;
                self.nodes[j].vx -= fx;
                self.nodes[j].vy -= fy;
            }
        }
    }

    /// 碰撞力，防止节点堆叠
    fn apply_collision_force(&mut self) {
        let n = self.nodes.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.nodes[j].x - self.nodes[i].x;
                let dy = self.nodes[j].y - self.nodes[i].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.1);
                let row_padding = if self.nodes[i].depth == self.nodes[j].depth {
                    self.collision_padding + 18.0
                } else {
                    self.collision_padding
                };
                let min_distance = self.nodes[i].radius + self.nodes[j].radius + row_padding;

                if distance >= min_distance {
                    continue;
                }

                let overlap = (min_distance - distance) / distance * 0.5 * self.alpha;
                let fx = dx * overlap;
                let fy = dy * overlap;

                self.nodes[i].vx -= fx;
                self.nodes[i].vy -= fy;
                self.nodes[j].vx += fx;
                self.nodes[j].vy += fy;
            }
        }
    }

    /// 锚点力，让每层节点停靠在预设位置
    fn apply_center_force(&mut self) {
        for node in &mut self.nodes {
            let depth_bias = 1.0 + node.depth as f32 * 0.08;
            let fx = (node.home_x - node.x) * self.center_strength * self.alpha;
            let fy = (node.home_y - node.y) * self.center_strength * depth_bias * self.alpha;
            node.vx += fx;
            node.vy += fy;
        }
    }

    /// 应用速度到位置
    fn apply_velocity(&mut self) {
        for node in &mut self.nodes {
            // 如果节点被固定，不移动
            if let (Some(fx), Some(fy)) = (node.fx, node.fy) {
                node.x = fx;
                node.y = fy;
                node.vx = 0.0;
                node.vy = 0.0;
                continue;
            }

            node.vx *= 1.0 - self.velocity_decay;
            node.vy *= 1.0 - self.velocity_decay;

            node.x += node.vx;
            node.y += node.vy;

            // 边界约束
            node.x = node.x.max(node.radius).min(self.width - node.radius);
            node.y = node.y.max(node.radius).min(self.height - node.radius);
        }
    }

    /// 固定节点位置
    pub fn fix_node(&mut self, node_id: u64, x: f32, y: f32) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.fx = Some(x);
            node.fy = Some(y);
            node.x = x;
            node.y = y;
            node.vx = 0.0;
            node.vy = 0.0;
        }
    }

    fn seed_by_depth(&mut self) {
        let mut levels: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            levels.entry(node.depth).or_default().push(index);
        }

        let max_depth = levels.keys().copied().max().unwrap_or(0).max(1);
        let top_padding = 96.0;
        let bottom_padding = 84.0;
        let vertical_gap = ((self.height - top_padding - bottom_padding) / max_depth as f32)
            .max(132.0);
        let horizontal_padding = 84.0;
        let usable_width = (self.width - horizontal_padding * 2.0).max(240.0);

        for (depth, indices) in levels {
            let y = top_padding + depth as f32 * vertical_gap;
            let count = indices.len();

            for (slot, index) in indices.into_iter().enumerate() {
                let spread = (slot + 1) as f32 / (count + 1) as f32;
                let jitter = ((index as f32 * 11.0) % 24.0) - 12.0;
                let home_x = horizontal_padding + usable_width * spread + jitter;
                let node = &mut self.nodes[index];
                node.home_x = home_x;
                node.home_y = y;
                node.x = home_x;
                node.y = y;
            }
        }
    }
}

/// 从组织树构建力导向图数据
pub fn build_force_data_from_tree(
    tree: &super::org_node::OrgNode,
) -> (Vec<ForceNode>, Vec<ForceLink>) {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    fn traverse(
        node: &super::org_node::OrgNode,
        nodes: &mut Vec<ForceNode>,
        links: &mut Vec<ForceLink>,
        depth: usize,
    ) {
        // 添加节点
        nodes.push(ForceNode::from_org(node, depth));

        if !node.expanded {
            return;
        }

        // 添加边和递归子节点
        for child in &node.children {
            links.push(ForceLink {
                source: node.id,
                target: child.id,
                distance: 180.0 + depth as f32 * 28.0 + child.children.len() as f32 * 12.0,
                weight: 0.8 + child.collaboration_index / 100.0,
            });
            traverse(child, nodes, links, depth + 1);
        }
    }

    traverse(tree, &mut nodes, &mut links, 0);
    (nodes, links)
}
