//! 组织地图画布组件
//!
//! 基于 SVG 的交互式组织架构可视化。

use super::force_simulation::{build_force_data_from_tree, ForceSimulation};
use super::org_node::{DragChangePreview, ForceLink, ForceNode, OrgNode, OrgNodeType, HISTORY_MONTHS};
use super::{AtlasViewMode, SortMode};
use dioxus::html::InteractionElementOffset;
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct CanvasOffset {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PanGesture {
    dragging: bool,
    last_x: f32,
    last_y: f32,
}

/// 节点拖拽状态
#[derive(Clone, Debug, Default, PartialEq)]
struct NodeDragState {
    /// 正在拖拽的节点 ID
    node_id: Option<u64>,
    /// 拖拽起始时节点的原始位置
    origin_x: f32,
    origin_y: f32,
    /// 当前拖拽偏移（SVG transform 空间）
    delta_x: f32,
    delta_y: f32,
    /// 拖拽中鼠标上一帧位置（SVG element coordinates）
    last_mouse_x: f32,
    last_mouse_y: f32,
    /// 命中的放置目标
    drop_target: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
struct LayoutSnapshot {
    nodes: Vec<ForceNode>,
    links: Vec<ForceLink>,
}

/// SVG 画布组件
#[component]
pub fn OrgCanvas(
    tree: OrgNode,
    width: f32,
    height: f32,
    mode: AtlasViewMode,
    timeline_index: usize,
    sort_mode: SortMode,
    selected_node_id: Option<u64>,
    on_node_click: Callback<u64>,
    on_node_toggle: Callback<u64>,
    on_drag_complete: Callback<DragChangePreview>,
) -> Element {
    let layout = use_memo(use_reactive((&tree, &width, &height, &sort_mode), |(tree, width, height, sort_mode)| {
        build_layout_snapshot(&tree, width, height, sort_mode)
    }));

    let mut zoom = use_signal(|| 1.0_f32);
    let mut offset = use_signal(CanvasOffset::default);
    let mut pan_gesture = use_signal(PanGesture::default);
    let mut node_drag = use_signal(NodeDragState::default);

    let layout = layout();
    let nodes = layout.nodes.clone();
    let links = layout.links.clone();
    let node_positions: HashMap<u64, (f32, f32)> =
        nodes.iter().map(|node| (node.id, (node.x, node.y))).collect();

    // Build a radius lookup for hit detection
    let node_radii: HashMap<u64, f32> = nodes.iter()
        .map(|n| (n.id, n.radius_at(timeline_index) + 12.0)) // +12 for easier drop targeting
        .collect();

    let drag = node_drag();
    let canvas_cursor = if drag.node_id.is_some() {
        "grabbing"
    } else if pan_gesture().dragging {
        "grabbing"
    } else {
        "grab"
    };
    let group_transform = format!(
        "translate({:.1} {:.1}) scale({:.3})",
        offset().x,
        offset().y,
        zoom()
    );
    let mode_title = match mode {
        AtlasViewMode::Topology => "拓扑视图",
        AtlasViewMode::Heatmap => "热力视图",
        AtlasViewMode::Timeline => "时间轴视图",
    };
    let mode_hint = match mode {
        AtlasViewMode::Topology => "节点大小表示人数，双击节点展开或收起下级结构。",
        AtlasViewMode::Heatmap => "颜色越暖表示增长越快，连线越粗表示协作越频繁。",
        AtlasViewMode::Timeline => "拖动时间轴回看过去 6 个月的组织规模变化。",
    };
    let timeline_label = HISTORY_MONTHS
        .get(timeline_index)
        .copied()
        .unwrap_or(HISTORY_MONTHS[HISTORY_MONTHS.len() - 1]);

    rsx! {
        div {
            style: format!(
                "width: 100%; max-width: 100%; min-width: 0; height: {height}px; background: linear-gradient(180deg, rgba(15, 23, 42, 0.92), rgba(11, 18, 32, 0.98)); border-radius: var(--ds-radius-lg); overflow: hidden; position: relative; border: 1px solid rgba(148, 163, 184, 0.14);",
            ),
            onwheel: move |event| {
                event.prevent_default();
                let delta_y = event.delta().strip_units().y;
                let factor = if delta_y < 0.0 { 1.08 } else { 0.92 };
                zoom.set((zoom() * factor as f32).clamp(0.55, 2.2));
            },
            onmousemove: move |event| {
                let coordinates = event.element_coordinates();
                let mx = coordinates.x as f32;
                let my = coordinates.y as f32;

                // Handle node drag
                let drag_state = node_drag();
                if let Some(_drag_id) = drag_state.node_id {
                    let z = zoom();
                    let o = offset();
                    // Convert element coordinates to graph coordinates
                    let graph_x = (mx - o.x) / z;
                    let graph_y = (my - o.y) / z;
                    let new_delta_x = graph_x - drag_state.origin_x;
                    let new_delta_y = graph_y - drag_state.origin_y;

                    // Hit detection: find closest node to current cursor position
                    let mut hit_target: Option<u64> = None;
                    for (id, (nx, ny)) in &node_positions {
                        if *id == _drag_id {
                            continue;
                        }
                        let r = node_radii.get(id).copied().unwrap_or(30.0);
                        let dx = graph_x - nx;
                        let dy = graph_y - ny;
                        if (dx * dx + dy * dy) < r * r {
                            hit_target = Some(*id);
                            break;
                        }
                    }
                    node_drag
                        .set(NodeDragState {
                            node_id: drag_state.node_id,
                            origin_x: drag_state.origin_x,
                            origin_y: drag_state.origin_y,
                            delta_x: new_delta_x,
                            delta_y: new_delta_y,
                            last_mouse_x: mx,
                            last_mouse_y: my,
                            drop_target: hit_target,
                        });
                    return;
                }
                let gesture = pan_gesture();
                if !gesture.dragging {
                    return;
                }
                offset
                    .with_mut(|offset| {
                        offset.x += mx - gesture.last_x;
                        offset.y += my - gesture.last_y;
                    });
                pan_gesture
                    .set(PanGesture {
                        dragging: true,
                        last_x: mx,
                        last_y: my,
                    });
            },
            onmouseup: move |_| {
                // Check if we were dragging a node
                let drag_state = node_drag();
                if let Some(drag_id) = drag_state.node_id {
                    if let Some(target_id) = drag_state.drop_target {
                        // Generate preview via tree
                        let preview = tree.preview_move(drag_id, target_id);
                        if let Some(p) = preview {
                            on_drag_complete.call(p);
                        }
                    }
                    node_drag.set(NodeDragState::default());
                }
                pan_gesture.set(PanGesture::default());
            },
            onmouseleave: move |_| {
                node_drag.set(NodeDragState::default());
                pan_gesture.set(PanGesture::default());
            },

            svg {
                width: "100%",
                height: "100%",
                view_box: "0 0 {width} {height}",
                style: format!("display: block; width: 100%; height: 100%; cursor: {canvas_cursor};"),
                onmousedown: move |event| {
                    let coordinates = event.element_coordinates();
                    pan_gesture
                        .set(PanGesture {
                            dragging: true,
                            last_x: coordinates.x as f32,
                            last_y: coordinates.y as f32,
                        });
                },

                defs {
                    radialGradient {
                        id: "company-gradient",
                        cx: "50%",
                        cy: "38%",
                        r: "70%",
                        stop { offset: "0%", stop_color: "#a7f3d0" }
                        stop { offset: "100%", stop_color: "#0f766e" }
                    }
                    filter {
                        id: "node-shadow",
                        x: "-20%",
                        y: "-20%",
                        width: "140%",
                        height: "140%",
                        feDropShadow {
                            dx: "0",
                            dy: "8",
                            std_deviation: "12",
                            flood_color: "#020617",
                            flood_opacity: "0.32",
                        }
                    }
                }

                g { transform: "{group_transform}",
                    for link in links.iter() {
                        if let Some((sx, sy)) = node_positions.get(&link.source) {
                            if let Some((tx, ty)) = node_positions.get(&link.target) {
                                LinkLine {
                                    x1: *sx,
                                    y1: *sy,
                                    x2: *tx,
                                    y2: *ty,
                                    stroke: link_color(mode, link.weight),
                                    stroke_width: link_width(mode, link.weight),
                                    opacity: link_opacity(mode),
                                }
                            }
                        }
                    }

                    for node in nodes.iter() {
                        {
                            let is_dragging = drag.node_id == Some(node.id);
                            let is_drop_target = drag.drop_target == Some(node.id);
                            let drag_delta = if is_dragging {
                                (drag.delta_x, drag.delta_y)
                            } else {
                                (0.0, 0.0)
                            };
                            let nid = node.id;
                            let node_x = node.x;
                            let node_y = node.y;

                            rsx! {
                                NodeCircle {
                                    node: node.clone(),
                                    mode,
                                    timeline_index,
                                    selected: selected_node_id == Some(node.id),
                                    is_dragging,
                                    is_drop_target,
                                    drag_delta,
                                    onclick: {
                                        let callback = on_node_click.clone();
                                        move || callback.call(nid)
                                    },
                                    ontoggle: {
                                        let callback = on_node_toggle.clone();
                                        move || callback.call(nid)
                                    },
                                    on_drag_start: move || {
                                        node_drag
                                            .set(NodeDragState {
                                                node_id: Some(nid),
                                                origin_x: node_x,
                                                origin_y: node_y,
                                                delta_x: 0.0,
                                                delta_y: 0.0,
                                                last_mouse_x: 0.0,
                                                last_mouse_y: 0.0,
                                                drop_target: None,
                                            });
                                        pan_gesture.set(PanGesture::default());
                                    },
                                }
                            }
                        }
                    }
                }
            }

            div { style: "position: absolute; top: 16px; left: 16px; display: flex; flex-direction: column; gap: 6px; padding: 12px 14px; border-radius: 14px; background: rgba(2, 6, 23, 0.58); border: 1px solid rgba(148, 163, 184, 0.14); backdrop-filter: blur(14px);",
                div { style: "font-size: 12px; font-weight: 600; color: rgba(226, 232, 240, 0.96); letter-spacing: 0.01em;",
                    "{mode_title}"
                }
                div { style: "font-size: 12px; line-height: 1.5; max-width: 280px; color: rgba(148, 163, 184, 0.92);",
                    "{mode_hint}"
                }
                if matches!(mode, AtlasViewMode::Timeline) {
                    div { style: "font-size: 11px; color: rgba(94, 234, 212, 0.9);",
                        "当前时间点：{timeline_label}"
                    }
                }
            }

            div { style: "position: absolute; bottom: 16px; left: 16px;",
                ModeLegend { mode, timeline_index }
            }

            div { style: "position: absolute; top: 16px; right: 16px; display: flex; flex-direction: column; gap: 8px;",
                ZoomButton {
                    label: "+",
                    title: "放大",
                    on_zoom: move || zoom.set((zoom() * 1.12).clamp(0.55, 2.2)),
                }
                ZoomButton {
                    label: "-",
                    title: "缩小",
                    on_zoom: move || zoom.set((zoom() * 0.9).clamp(0.55, 2.2)),
                }
                ZoomButton {
                    label: "R",
                    title: "重置",
                    on_zoom: move || {
                        zoom.set(1.0);
                        offset.set(CanvasOffset::default());
                    },
                }
            }
        }
    }
}

#[component]
fn NodeCircle(
    node: ForceNode,
    mode: AtlasViewMode,
    timeline_index: usize,
    selected: bool,
    is_dragging: bool,
    is_drop_target: bool,
    drag_delta: (f32, f32),
    onclick: EventHandler<()>,
    ontoggle: EventHandler<()>,
    on_drag_start: EventHandler<()>,
) -> Element {
    let radius = node.radius_at(timeline_index);
    let member_count = node.headcount_at(timeline_index);
    let label_size = if radius > 38.0 { 13 } else { 11 };
    let glow_radius = radius + if selected { 10.0 } else { 7.0 };
    let stroke_width = if selected { 3.0 } else { 1.5 };
    let fill = node_fill(&node, mode, timeline_index);
    let halo_opacity = if selected { "0.72" } else { "0.24" };
    let stroke = if selected {
        "rgba(110, 231, 183, 0.98)"
    } else {
        "rgba(15, 23, 42, 0.9)"
    };
    let text_color = match mode {
        AtlasViewMode::Heatmap if node.growth_rate < -2.0 => "#f8fafc",
        AtlasViewMode::Topology if matches!(node.node_type, OrgNodeType::Company) => "#ecfeff",
        _ => "#e2e8f0",
    };
    let subtitle = node_subtitle(&node, mode, timeline_index, member_count);
    let collapse_marker = if node.expanded { "-" } else { "+" };

    // Drag visual: translate the dragged node, show drop target glow
    let transform = if is_dragging {
        format!("translate({:.1} {:.1})", drag_delta.0, drag_delta.1)
    } else {
        String::new()
    };
    let drag_opacity = if is_dragging { "0.85" } else { "1.0" };

    // Drop target: pulsing green ring
    let drop_target_stroke = if is_drop_target {
        "rgba(52, 211, 153, 0.95)"
    } else {
        stroke.as_ref()
    };
    let drop_target_stroke_width = if is_drop_target { 4.0 } else { stroke_width };
    let drop_target_halo_radius = if is_drop_target {
        radius + 16.0
    } else {
        glow_radius
    };

    rsx! {
        g {
            style: "cursor: pointer;",
            transform: "{transform}",
            opacity: "{drag_opacity}",
            onmousedown: move |event| {
                event.stop_propagation();
                on_drag_start.call(());
            },
            onclick: move |_| onclick.call(()),
            ondoubleclick: move |_| ontoggle.call(()),

            // Outer halo / drop target ring
            circle {
                cx: "{node.x}",
                cy: "{node.y}",
                r: "{drop_target_halo_radius}",
                fill: if is_drop_target { "rgba(52, 211, 153, 0.12)" } else { "none" },
                stroke: if is_drop_target { "rgba(52, 211, 153, 0.95)" } else { selected_halo(mode, node.growth_rate) },
                stroke_width: if is_drop_target { "3" } else { "2" },
                opacity: if is_drop_target { "0.9" } else { "{halo_opacity}" },
            }

            circle {
                cx: "{node.x}",
                cy: "{node.y}",
                r: "{radius}",
                fill: "{fill}",
                stroke: "{drop_target_stroke}",
                stroke_width: "{drop_target_stroke_width}",
                filter: "url(#node-shadow)",
            }

            text {
                x: "{node.x}",
                y: "{node.y - 4.0}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                fill: "{text_color}",
                font_size: "{label_size}",
                font_weight: "600",
                pointer_events: "none",
                "{node.name}"
            }

            text {
                x: "{node.x}",
                y: "{node.y + 13.0}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                fill: "rgba(226, 232, 240, 0.82)",
                font_size: "10",
                pointer_events: "none",
                "{subtitle}"
            }

            if node.child_count > 0 {
                circle {
                    cx: "{node.x + radius * 0.65}",
                    cy: "{node.y - radius * 0.65}",
                    r: "10",
                    fill: "rgba(15, 23, 42, 0.96)",
                    stroke: "rgba(148, 163, 184, 0.3)",
                    stroke_width: "1",
                }
                text {
                    x: "{node.x + radius * 0.65}",
                    y: "{node.y - radius * 0.65 + 1.0}",
                    text_anchor: "middle",
                    dominant_baseline: "middle",
                    fill: "#e2e8f0",
                    font_size: "11",
                    font_weight: "700",
                    pointer_events: "none",
                    "{collapse_marker}"
                }
            }
        }
    }
}

#[component]
fn LinkLine(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    stroke: String,
    stroke_width: f32,
    opacity: f32,
) -> Element {
    rsx! {
        line {
            x1: "{x1}",
            y1: "{y1}",
            x2: "{x2}",
            y2: "{y2}",
            stroke: "{stroke}",
            stroke_width: "{stroke_width}",
            opacity: "{opacity}",
        }
    }
}

#[component]
fn ZoomButton(
    label: &'static str,
    title: &'static str,
    on_zoom: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            title: "{title}",
            onclick: move |_| on_zoom.call(()),
            style: "width: 34px; height: 34px; border-radius: 10px; background: rgba(15, 23, 42, 0.82); border: 1px solid rgba(148, 163, 184, 0.16); color: rgba(226, 232, 240, 0.92); font-size: 14px; cursor: pointer; transition: all var(--ds-transition-fast); backdrop-filter: blur(14px);",
            "{label}"
        }
    }
}

#[component]
fn ModeLegend(
    mode: AtlasViewMode,
    timeline_index: usize,
) -> Element {
    let description = match mode {
        AtlasViewMode::Topology => "圆越大人数越多，双击可展开或收起分支。".to_string(),
        AtlasViewMode::Heatmap => "绿色扩张，橙红收缩，连线宽度表示跨团队协作热度。".to_string(),
        AtlasViewMode::Timeline => HISTORY_MONTHS
            .get(timeline_index)
            .map(|label| format!("正在查看 {label} 的组织规模快照。"))
            .unwrap_or_else(|| "正在查看组织历史快照。".to_string()),
    };

    rsx! {
        div { style: "display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 12px; background: rgba(2, 6, 23, 0.54); border: 1px solid rgba(148, 163, 184, 0.12); backdrop-filter: blur(12px);",
            div { style: "display: flex; gap: 6px;",
                div { style: "width: 10px; height: 10px; border-radius: 999px; background: #34d399;" }
                div { style: "width: 10px; height: 10px; border-radius: 999px; background: #f59e0b;" }
                div { style: "width: 10px; height: 10px; border-radius: 999px; background: #fb7185;" }
            }
            div { style: "font-size: 11px; color: rgba(203, 213, 225, 0.88);", "{description}" }
        }
    }
}

fn build_layout_snapshot(tree: &OrgNode, width: f32, height: f32, sort_mode: SortMode) -> LayoutSnapshot {
    let (nodes, links) = build_force_data_from_tree(tree, sort_mode);
    let mut simulation = ForceSimulation::new(width, height);
    simulation.set_nodes(nodes);
    simulation.set_links(links);
    simulation.fix_node(tree.id, width / 2.0, 96.0);
    simulation.run(420);

    LayoutSnapshot {
        nodes: simulation.nodes().to_vec(),
        links: simulation.links().to_vec(),
    }
}

fn link_color(mode: AtlasViewMode, weight: f32) -> String {
    match mode {
        AtlasViewMode::Topology => "rgba(148, 163, 184, 0.42)".to_string(),
        AtlasViewMode::Heatmap => {
            if weight >= 1.55 {
                "rgba(52, 211, 153, 0.72)".to_string()
            } else if weight >= 1.3 {
                "rgba(250, 204, 21, 0.68)".to_string()
            } else {
                "rgba(244, 114, 182, 0.58)".to_string()
            }
        }
        AtlasViewMode::Timeline => "rgba(56, 189, 248, 0.36)".to_string(),
    }
}

fn link_width(mode: AtlasViewMode, weight: f32) -> f32 {
    match mode {
        AtlasViewMode::Topology => 1.4 + weight * 0.85,
        AtlasViewMode::Heatmap => 1.6 + weight * 1.25,
        AtlasViewMode::Timeline => 1.2 + weight * 0.7,
    }
}

fn link_opacity(mode: AtlasViewMode) -> f32 {
    match mode {
        AtlasViewMode::Topology => 0.68,
        AtlasViewMode::Heatmap => 0.82,
        AtlasViewMode::Timeline => 0.54,
    }
}

fn node_fill(node: &ForceNode, mode: AtlasViewMode, timeline_index: usize) -> String {
    match mode {
        AtlasViewMode::Topology => match node.node_type {
            OrgNodeType::Company => "url(#company-gradient)".to_string(),
            OrgNodeType::Department => "#0f766e".to_string(),
            OrgNodeType::Group => "#164e63".to_string(),
        },
        AtlasViewMode::Heatmap => growth_color(node.growth_rate).to_string(),
        AtlasViewMode::Timeline => {
            let headcount = node.headcount_at(timeline_index) as f32;
            let intensity = (headcount / 60.0).clamp(0.18, 1.0);
            let lightness = 62.0 - intensity * 20.0;
            format!("hsl(191 70% {lightness:.0}%)")
        }
    }
}

fn node_subtitle(
    node: &ForceNode,
    mode: AtlasViewMode,
    timeline_index: usize,
    member_count: u32,
) -> String {
    match mode {
        AtlasViewMode::Topology => format!("{member_count} 人"),
        AtlasViewMode::Heatmap => format!("{:+.1}% 增长", node.growth_rate),
        AtlasViewMode::Timeline => {
            let label = HISTORY_MONTHS
                .get(timeline_index)
                .copied()
                .unwrap_or(HISTORY_MONTHS[HISTORY_MONTHS.len() - 1]);
            format!("{label} · {member_count} 人")
        }
    }
}

fn growth_color(growth_rate: f32) -> &'static str {
    if growth_rate >= 8.0 {
        "#22c55e"
    } else if growth_rate >= 3.0 {
        "#84cc16"
    } else if growth_rate >= 0.0 {
        "#f59e0b"
    } else if growth_rate >= -3.0 {
        "#f97316"
    } else {
        "#fb7185"
    }
}

fn selected_halo(mode: AtlasViewMode, growth_rate: f32) -> &'static str {
    match mode {
        AtlasViewMode::Topology => "rgba(94, 234, 212, 0.9)",
        AtlasViewMode::Heatmap => growth_color(growth_rate),
        AtlasViewMode::Timeline => "rgba(56, 189, 248, 0.82)",
    }
}
