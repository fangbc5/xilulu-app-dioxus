//! Atlas 组织地图视图
//!
//! 可视化的组织架构浏览界面。

pub mod canvas;
pub mod force_simulation;
pub mod org_node;

use crate::components::page_header::PageHeader;
use crate::views::atlas::canvas::OrgCanvas;
use crate::views::atlas::org_node::{OrgNode, OrgNodeType, HISTORY_MONTHS};
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtlasViewMode {
    Topology,
    Heatmap,
    Timeline,
}

/// AtlasView 组织地图视图
#[component]
pub fn AtlasView() -> Element {
    let mut tree = use_signal(build_mock_org_tree);
    let mut selected_node_id = use_signal(|| Some(1_u64));
    let mut view_mode = use_signal(|| AtlasViewMode::Topology);
    let timeline_index = use_signal(|| HISTORY_MONTHS.len() - 1);
    let navigator = use_navigator();

    let (selected_node, visible_nodes, expanded_branches, total_people, portfolio_growth) = {
        let tree_ref = tree.read();
        let selected = selected_node_id()
            .and_then(|node_id| tree_ref.find(node_id).cloned())
            .or_else(|| Some(tree_ref.clone()));
        let visible_nodes = tree_ref.visible_node_count();
        let expanded_branches = count_expanded_branches(&tree_ref);
        let total_people = tree_ref.headcount_at(timeline_index());
        let portfolio_growth = average_growth(&tree_ref);

        (
            selected,
            visible_nodes,
            expanded_branches,
            total_people,
            portfolio_growth,
        )
    };

    rsx! {
        div {
            class: "atlas-page",
            style: "width: 100%;",

            div {
                class: "atlas-shell",
                style: "
                    width: min(100%, 1380px);
                    margin: 0 auto;
                    display: flex;
                    flex-direction: column;
                    gap: 20px;
                ",

                PageHeader {
                    title: "组织地图".to_string(),
                    description: Some("力导向 Atlas，支持拓扑、热力和时间轴回看。".to_string()),
                }

                div {
                    class: "atlas-toolbar",
                    style: "
                        display: flex;
                        flex-wrap: wrap;
                        align-items: center;
                        justify-content: space-between;
                        gap: 14px 18px;
                    ",
                    div {
                        class: "atlas-toolbar-left",
                        style: "display: flex; flex-wrap: wrap; gap: 10px;",
                        ViewModeChip {
                            label: "拓扑视图",
                            active: view_mode() == AtlasViewMode::Topology,
                            onclick: move || view_mode.set(AtlasViewMode::Topology),
                        }
                        ViewModeChip {
                            label: "热力视图",
                            active: view_mode() == AtlasViewMode::Heatmap,
                            onclick: move || view_mode.set(AtlasViewMode::Heatmap),
                        }
                        ViewModeChip {
                            label: "时间轴",
                            active: view_mode() == AtlasViewMode::Timeline,
                            onclick: move || view_mode.set(AtlasViewMode::Timeline),
                        }
                    }

                    div {
                        class: "atlas-toolbar-right",
                        style: "
                            display: flex;
                            flex-wrap: wrap;
                            justify-content: flex-end;
                            gap: 10px;
                        ",
                        SummaryPill {
                            label: "可见节点",
                            value: visible_nodes.to_string(),
                            accent: "var(--ds-accent)".to_string(),
                        }
                        SummaryPill {
                            label: "在岗人数",
                            value: format!("{total_people} 人"),
                            accent: "#38bdf8".to_string(),
                        }
                        SummaryPill {
                            label: "展开分支",
                            value: expanded_branches.to_string(),
                            accent: "#f59e0b".to_string(),
                        }
                        SummaryPill {
                            label: "组合增长",
                            value: format!("{portfolio_growth:+.1}%"),
                            accent: growth_accent(portfolio_growth).to_string(),
                        }
                    }
                }

                if view_mode() == AtlasViewMode::Timeline {
                    TimelineControl {
                        current: timeline_index,
                        labels: HISTORY_MONTHS.iter().map(|label| label.to_string()).collect(),
                    }
                }

                div {
                    class: "atlas-workspace",
                    style: "
                        display: grid;
                        grid-template-columns: minmax(0, 1fr) 340px;
                        gap: 24px;
                        align-items: start;
                    ",
                    div {
                        class: "atlas-stage",
                        style: "display: flex; min-width: 0;",
                        OrgCanvas {
                            tree: tree(),
                            width: 1080.0,
                            height: 620.0,
                            mode: view_mode(),
                            timeline_index: timeline_index(),
                            selected_node_id: selected_node_id(),
                            on_node_click: move |node_id| selected_node_id.set(Some(node_id)),
                            on_node_toggle: move |node_id| {
                                tree.with_mut(|tree| {
                                    tree.toggle_expanded(node_id);
                                });
                                selected_node_id.set(Some(node_id));
                            },
                        }
                    }

                    div {
                        class: "atlas-inspector",
                        style: "display: flex; flex-direction: column; gap: 16px; min-width: 0;",
                        InspectorCard {
                            title: "当前视图".to_string(),
                            eyebrow: "Atlas".to_string(),
                            body: match view_mode() {
                                AtlasViewMode::Topology => "查看组织拓扑，双击节点可展开或收起分支。".to_string(),
                                AtlasViewMode::Heatmap => "用热度颜色定位扩张、收缩和协作密集的组织区域。".to_string(),
                                AtlasViewMode::Timeline => {
                                    let label = HISTORY_MONTHS
                                        .get(timeline_index())
                                        .copied()
                                        .unwrap_or(HISTORY_MONTHS[HISTORY_MONTHS.len() - 1]);
                                    format!("时间轴已切换到 {label}，节点大小会按当月人数回放。")
                                }
                            }
                        }

                        if let Some(node) = selected_node {
                            SelectedNodePanel {
                                node: node.clone(),
                                mode: view_mode(),
                                timeline_index: timeline_index(),
                                on_toggle_expand: {
                                    let node_id = node.id;
                                    move || {
                                        tree.with_mut(|tree| {
                                            tree.toggle_expanded(node_id);
                                        });
                                    }
                                },
                                on_open_people: move || {
                                    navigator.push("/people");
                                },
                                on_focus_timeline: move || view_mode.set(AtlasViewMode::Timeline),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ViewModeChip(
    label: &'static str,
    active: bool,
    onclick: EventHandler<()>,
) -> Element {
    let background = if active {
        "rgba(16, 185, 129, 0.14)"
    } else {
        "var(--ds-bg-surface)"
    };
    let color = if active {
        "var(--ds-accent)"
    } else {
        "var(--ds-text-secondary)"
    };
    let border = if active {
        "rgba(16, 185, 129, 0.28)"
    } else {
        "var(--ds-border)"
    };

    rsx! {
        button {
            onclick: move |_| onclick.call(()),
            style: format!(
                "padding: 8px 14px; border-radius: 999px; font-size: 13px; cursor: pointer; transition: all var(--ds-transition-fast); background: {background}; color: {color}; border: 1px solid {border};"
            ),
            "{label}"
        }
    }
}

#[component]
fn SummaryPill(
    label: &'static str,
    value: String,
    accent: String,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 14px; background: var(--ds-bg-surface); border: 1px solid var(--ds-border);",
            div {
                style: format!("width: 8px; height: 8px; border-radius: 999px; background: {accent};")
            }
            div {
                style: "display: flex; flex-direction: column; gap: 2px;",
                div {
                    style: "font-size: 11px; color: var(--ds-text-tertiary);",
                    "{label}"
                }
                div {
                    style: "font-size: 13px; font-weight: 600; color: var(--ds-text-primary);",
                    "{value}"
                }
            }
        }
    }
}

#[component]
fn TimelineControl(
    current: Signal<usize>,
    labels: Vec<String>,
) -> Element {
    let max = labels.len().saturating_sub(1);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 12px; padding: 14px 16px; border-radius: var(--ds-radius-md); background: var(--ds-bg-surface); border: 1px solid var(--ds-border);",
            div {
                style: "display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                div {
                    style: "font-size: 13px; font-weight: 600; color: var(--ds-text-primary);",
                    "组织时间轴"
                }
                div {
                    style: "font-size: 12px; color: var(--ds-accent);",
                    "{labels.get(current()).cloned().unwrap_or_default()}"
                }
            }

            input {
                r#type: "range",
                min: "0",
                max: "{max}",
                step: "1",
                value: "{current}",
                oninput: move |event| {
                    if let Ok(value) = event.value().parse::<usize>() {
                        current.set(value.min(max));
                    }
                },
            }

            div {
                style: "display: flex; justify-content: space-between; gap: 8px; flex-wrap: wrap;",
                for (index, label) in labels.iter().enumerate() {
                    button {
                            onclick: move |_| current.set(index),
                        style: format!(
                            "padding: 4px 8px; border-radius: 999px; font-size: 11px; border: 1px solid {}; background: {}; color: {}; cursor: pointer;",
                            if current() == index { "rgba(16, 185, 129, 0.28)" } else { "var(--ds-border)" },
                            if current() == index { "rgba(16, 185, 129, 0.12)" } else { "transparent" },
                            if current() == index { "var(--ds-accent)" } else { "var(--ds-text-tertiary)" },
                        ),
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
fn InspectorCard(
    eyebrow: String,
    title: String,
    body: String,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 10px; padding: 18px; border-radius: 18px; background: var(--ds-bg-surface); border: 1px solid var(--ds-border);",
            div {
                style: "font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--ds-text-tertiary);",
                "{eyebrow}"
            }
            div {
                style: "font-size: 16px; font-weight: 600; color: var(--ds-text-primary);",
                "{title}"
            }
            div {
                style: "font-size: 13px; line-height: 1.6; color: var(--ds-text-secondary);",
                "{body}"
            }
        }
    }
}

#[component]
fn SelectedNodePanel(
    node: OrgNode,
    mode: AtlasViewMode,
    timeline_index: usize,
    on_toggle_expand: EventHandler<()>,
    on_open_people: EventHandler<()>,
    on_focus_timeline: EventHandler<()>,
) -> Element {
    let current_count = node.headcount_at(timeline_index);
    let node_type = match node.node_type {
        OrgNodeType::Company => "公司",
        OrgNodeType::Department => "部门",
        OrgNodeType::Group => "小组",
    };
    let expand_label = if node.children.is_empty() {
        "没有下级结构"
    } else if node.expanded {
        "收起下级结构"
    } else {
        "展开下级结构"
    };
    let can_expand = !node.children.is_empty();
    let chart_max = node.history.iter().copied().max().unwrap_or(1) as f32;
    let month_label = HISTORY_MONTHS
        .get(timeline_index)
        .copied()
        .unwrap_or(HISTORY_MONTHS[HISTORY_MONTHS.len() - 1]);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 16px; padding: 20px; border-radius: 18px; background: linear-gradient(180deg, rgba(15, 23, 42, 0.98), rgba(15, 23, 42, 0.9)); border: 1px solid rgba(148, 163, 184, 0.14);",
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                div {
                    style: "font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: rgba(148, 163, 184, 0.82);",
                    "{node_type}"
                }
                div {
                    style: "font-size: 20px; font-weight: 600; color: #f8fafc;",
                    "{node.name}"
                }
                div {
                    style: "font-size: 13px; color: rgba(148, 163, 184, 0.92);",
                    "当前查看 {month_label} · {current_count} 人"
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px;",
                MetricCard {
                    label: "增长率",
                    value: format!("{:+.1}%", node.growth_rate),
                    accent: growth_accent(node.growth_rate).to_string(),
                }
                MetricCard {
                    label: "协作热度",
                    value: format!("{:.0}", node.collaboration_index),
                    accent: "#38bdf8".to_string(),
                }
                MetricCard {
                    label: "下级数量",
                    value: node.children.len().to_string(),
                    accent: "#f59e0b".to_string(),
                }
                MetricCard {
                    label: "节点状态",
                    value: if node.expanded { "已展开".to_string() } else { "已收起".to_string() },
                    accent: "var(--ds-accent)".to_string(),
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 10px;",
                div {
                    style: "font-size: 12px; font-weight: 600; color: rgba(226, 232, 240, 0.9);",
                    "规模变化"
                }
                div {
                    style: "display: flex; align-items: flex-end; gap: 8px; height: 96px;",
                    for (index, count) in node.history.iter().enumerate() {
                        div {
                            style: "display: flex; flex-direction: column; align-items: center; gap: 6px; flex: 1;",
                            div {
                                style: format!(
                                    "width: 100%; min-height: 16px; height: {:.1}px; border-radius: 10px 10px 4px 4px; background: {};",
                                    16.0 + (*count as f32 / chart_max) * 58.0,
                                    if index == timeline_index {
                                        "linear-gradient(180deg, rgba(52, 211, 153, 0.95), rgba(15, 118, 110, 0.98))"
                                    } else {
                                        "linear-gradient(180deg, rgba(56, 189, 248, 0.42), rgba(30, 41, 59, 0.9))"
                                    }
                                )
                            }
                            div {
                                style: "font-size: 10px; color: rgba(148, 163, 184, 0.78);",
                                "{count}"
                            }
                            div {
                                style: "font-size: 10px; color: rgba(148, 163, 184, 0.62);",
                                "{HISTORY_MONTHS[index]}"
                            }
                        }
                    }
                }
            }

            if !node.children.is_empty() {
                div {
                    style: "display: flex; flex-direction: column; gap: 10px;",
                    div {
                        style: "font-size: 12px; font-weight: 600; color: rgba(226, 232, 240, 0.9);",
                        "下级结构"
                    }
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        for child in node.children.iter() {
                            div {
                                style: "display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 12px; border-radius: 12px; background: rgba(15, 23, 42, 0.42); border: 1px solid rgba(148, 163, 184, 0.12);",
                                div {
                                    style: "display: flex; flex-direction: column; gap: 2px;",
                                    div {
                                        style: "font-size: 13px; color: #e2e8f0;",
                                        "{child.name}"
                                    }
                                    div {
                                        style: "font-size: 11px; color: rgba(148, 163, 184, 0.82);",
                                        match child.node_type {
                                            OrgNodeType::Company => "公司",
                                            OrgNodeType::Department => "部门",
                                            OrgNodeType::Group => "小组",
                                        }
                                    }
                                }
                                div {
                                    style: "font-size: 12px; color: rgba(94, 234, 212, 0.92);",
                                    "{child.member_count} 人"
                                }
                            }
                        }
                    }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 10px;",
                button {
                    disabled: !can_expand,
                    onclick: move |_| {
                        if can_expand {
                            on_toggle_expand.call(());
                        }
                    },
                    style: format!(
                        "padding: 8px 12px; border-radius: 12px; border: 1px solid {}; background: {}; color: {}; cursor: {};",
                        if can_expand { "rgba(16, 185, 129, 0.3)" } else { "rgba(71, 85, 105, 0.4)" },
                        if can_expand { "rgba(16, 185, 129, 0.14)" } else { "rgba(30, 41, 59, 0.45)" },
                        if can_expand { "#d1fae5" } else { "rgba(148, 163, 184, 0.72)" },
                        if can_expand { "pointer" } else { "not-allowed" },
                    ),
                    "{expand_label}"
                }
                button {
                    onclick: move |_| on_focus_timeline.call(()),
                    style: "padding: 8px 12px; border-radius: 12px; border: 1px solid rgba(56, 189, 248, 0.26); background: rgba(56, 189, 248, 0.1); color: #bae6fd; cursor: pointer;",
                    "切到时间轴"
                }
                button {
                    onclick: move |_| on_open_people.call(()),
                    style: "padding: 8px 12px; border-radius: 12px; border: 1px solid rgba(148, 163, 184, 0.18); background: rgba(15, 23, 42, 0.42); color: #e2e8f0; cursor: pointer;",
                    "查看人员流"
                }
            }

            div {
                style: "font-size: 12px; line-height: 1.6; color: rgba(148, 163, 184, 0.82);",
                match mode {
                    AtlasViewMode::Topology => "建议从顶层部门逐步展开，先看拓扑，再切到热力视图确认增长异动。".to_string(),
                    AtlasViewMode::Heatmap => "热力视图适合找增员过快或收缩明显的组织，再结合人员流做详细排查。".to_string(),
                    AtlasViewMode::Timeline => "时间轴会回放人数快照，适合和组织变更审批记录一起核对。".to_string(),
                }
            }
        }
    }
}

#[component]
fn MetricCard(
    label: &'static str,
    value: String,
    accent: String,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; padding: 12px; border-radius: 14px; background: rgba(15, 23, 42, 0.45); border: 1px solid rgba(148, 163, 184, 0.12);",
            div {
                style: "font-size: 11px; color: rgba(148, 163, 184, 0.82);",
                "{label}"
            }
            div {
                style: format!("font-size: 16px; font-weight: 600; color: {accent};"),
                "{value}"
            }
        }
    }
}

fn build_mock_org_tree() -> OrgNode {
    let mut root = OrgNode::root(1, "Xilulu Inc");

    let mut tech_center = branch_node(2, "技术中心", Some(1), true, 78.0);
    tech_center.add_child(group_node(5, "AI实验室", 2, 86.0, vec![9, 10, 10, 11, 11, 12]));
    tech_center.add_child(group_node(6, "前端组", 2, 72.0, vec![14, 15, 16, 16, 17, 18]));
    tech_center.add_child(group_node(7, "后端组", 2, 74.0, vec![12, 12, 13, 14, 15, 15]));

    let mut product = branch_node(3, "产品部", Some(1), true, 69.0);
    product.add_child(group_node(8, "产品组", 3, 66.0, vec![8, 9, 9, 10, 10, 10]));
    product.add_child(group_node(9, "设计组", 3, 71.0, vec![7, 7, 8, 9, 9, 10]));

    let mut marketing = branch_node(4, "市场部", Some(1), true, 64.0);
    marketing.add_child(group_node(10, "运营组", 4, 62.0, vec![11, 12, 13, 14, 14, 15]));
    marketing.add_child(group_node(11, "销售组", 4, 59.0, vec![9, 9, 9, 10, 10, 10]));

    let finance = OrgNode::leaf(
        12,
        "财务部",
        OrgNodeType::Department,
        Some(1),
        8,
        vec![7, 7, 8, 8, 8, 8],
        52.0,
    );

    root.add_child(tech_center);
    root.add_child(product);
    root.add_child(marketing);
    root.add_child(finance);
    root.refresh_rollup_metrics();
    root
}

fn branch_node(
    id: u64,
    name: &str,
    parent_id: Option<u64>,
    expanded: bool,
    collaboration_index: f32,
) -> OrgNode {
    OrgNode {
        id,
        name: name.to_string(),
        node_type: OrgNodeType::Department,
        parent_id,
        children: Vec::new(),
        member_count: 0,
        expanded,
        history: Vec::new(),
        growth_rate: 0.0,
        collaboration_index,
    }
}

fn group_node(
    id: u64,
    name: &str,
    parent_id: u64,
    collaboration_index: f32,
    history: Vec<u32>,
) -> OrgNode {
    let member_count = history.last().copied().unwrap_or_default();
    OrgNode::leaf(
        id,
        name,
        OrgNodeType::Group,
        Some(parent_id),
        member_count,
        history,
        collaboration_index,
    )
}

fn count_expanded_branches(node: &OrgNode) -> usize {
    let current = usize::from(!node.children.is_empty() && node.expanded);
    current
        + node
            .children
            .iter()
            .map(count_expanded_branches)
            .sum::<usize>()
}

fn average_growth(root: &OrgNode) -> f32 {
    let mut growths = Vec::new();
    collect_growth(root, &mut growths);

    if growths.is_empty() {
        0.0
    } else {
        growths.iter().sum::<f32>() / growths.len() as f32
    }
}

fn collect_growth(node: &OrgNode, growths: &mut Vec<f32>) {
    if matches!(node.node_type, OrgNodeType::Department | OrgNodeType::Group) {
        growths.push(node.growth_rate);
    }

    for child in &node.children {
        collect_growth(child, growths);
    }
}

fn growth_accent(value: f32) -> &'static str {
    if value >= 5.0 {
        "#4ade80"
    } else if value >= 0.0 {
        "#fbbf24"
    } else {
        "#fb7185"
    }
}
