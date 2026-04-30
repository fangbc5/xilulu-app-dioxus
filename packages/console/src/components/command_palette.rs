//! ⌘K 命令面板组件
//!
//! 全局快捷键触发的命令搜索面板，聚合所有模块注册的命令。
//! 支持模糊搜索、键盘导航、分组展示。

use crate::module::{Command, CommandAction, CommandCategory};
use dioxus::prelude::*;

/// 命令面板全局状态
#[derive(Clone, Copy)]
pub struct CommandPaletteState {
    pub is_open: Signal<bool>,
}

/// 命令面板 Provider
///
/// 在应用顶层挂载，提供全局快捷键监听和命令面板 UI。
#[component]
pub fn CommandPaletteProvider(
    commands: Vec<Command>,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| false);

    use_context_provider(|| CommandPaletteState { is_open });

    // 通过 JS 注册 document 级键盘事件，解决 div 焦点问题
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                return new Promise((resolve) => {
                    document.addEventListener('keydown', (e) => {
                        if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                            e.preventDefault();
                            dioxus.send("toggle");
                        }
                    });
                });
                "#,
            );
            // 持续接收 JS 发来的 toggle 事件
            while let Ok(_msg) = eval.recv::<String>().await {
                is_open.set(!is_open());
            }
        });
    });

    let commands_clone = commands.clone();

    rsx! {
        {children}
        if is_open() {
            CommandPaletteOverlay {
                commands: commands_clone.clone(),
                on_close: move |_| is_open.set(false),
            }
        }
    }
}

/// 命令面板浮层
#[component]
fn CommandPaletteOverlay(
    commands: Vec<Command>,
    on_close: EventHandler<()>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut selected_index = use_signal(|| 0usize);
    let nav = use_navigator();

    // 过滤命令
    let q = query().to_lowercase();
    let filtered: Vec<Command> = if q.is_empty() {
        commands.clone()
    } else {
        commands
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&q)
                    || c.description.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    };

    // 分组
    let nav_cmds: Vec<Command> = filtered
        .iter()
        .filter(|c| c.category == CommandCategory::Navigation)
        .cloned()
        .collect();
    let action_cmds: Vec<Command> = filtered
        .iter()
        .filter(|c| c.category == CommandCategory::Action)
        .cloned()
        .collect();

    let total = filtered.len();
    let nav_count = nav_cmds.len();

    rsx! {
        div {
            class: "command-overlay",
            onclick: move |_| on_close.call(()),
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    on_close.call(());
                }
            },

            div {
                class: "command-panel",
                onclick: move |e| e.stop_propagation(),

                // 搜索输入
                input {
                    class: "command-input",
                    r#type: "text",
                    placeholder: "输入命令或搜索...",
                    autofocus: true,
                    value: "{query}",
                    oninput: move |e| {
                        query.set(e.value());
                        selected_index.set(0);
                    },
                    onkeydown: move |e| {
                        match e.key() {
                            Key::ArrowDown => {
                                e.prevent_default();
                                let cur = selected_index();
                                if cur + 1 < total {
                                    selected_index.set(cur + 1);
                                }
                            }
                            Key::ArrowUp => {
                                e.prevent_default();
                                let cur = selected_index();
                                if cur > 0 {
                                    selected_index.set(cur - 1);
                                }
                            }
                            Key::Enter => {
                                if let Some(cmd) = filtered.get(selected_index()) {
                                    match &cmd.action {
                                        CommandAction::Navigate(path) => {
                                            nav.push(path.as_str());
                                        }
                                        CommandAction::OpenPanel(_) => {}
                                    }
                                    on_close.call(());
                                }
                            }
                            Key::Escape => {
                                on_close.call(());
                            }
                            _ => {}
                        }
                    },
                }

                // 命令列表
                div {
                    class: "command-list",

                    // 导航分组
                    if !nav_cmds.is_empty() {
                        div { class: "command-group-label", "导航" }
                        for (i, cmd) in nav_cmds.iter().enumerate() {
                            {
                                let is_sel = i == selected_index();
                                let sel_class = if is_sel { " selected" } else { "" };
                                let action = cmd.action.clone();
                                let name = cmd.name.clone();
                                let desc = cmd.description.clone();
                                rsx! {
                                    div {
                                        class: "command-item{sel_class}",
                                        onclick: {
                                            let action = action.clone();
                                            move |_| {
                                                if let CommandAction::Navigate(ref path) = action {
                                                    nav.push(path.as_str());
                                                }
                                                on_close.call(());
                                            }
                                        },
                                        span { class: "cmd-icon", "⬡" }
                                        span { class: "cmd-name", "{name}" }
                                        span { class: "cmd-desc", "{desc}" }
                                    }
                                }
                            }
                        }
                    }

                    // 操作分组
                    if !action_cmds.is_empty() {
                        div { class: "command-group-label", "操作" }
                        for (i, cmd) in action_cmds.iter().enumerate() {
                            {
                                let global_i = nav_count + i;
                                let is_sel = global_i == selected_index();
                                let sel_class = if is_sel { " selected" } else { "" };
                                let name = cmd.name.clone();
                                let desc = cmd.description.clone();
                                rsx! {
                                    div {
                                        class: "command-item{sel_class}",
                                        span { class: "cmd-icon", "◇" }
                                        span { class: "cmd-name", "{name}" }
                                        span { class: "cmd-desc", "{desc}" }
                                    }
                                }
                            }
                        }
                    }

                    // 空结果
                    if filtered.is_empty() {
                        div {
                            style: "padding: 24px; text-align: center; color: var(--ds-text-tertiary); font-size: 14px;",
                            "没有匹配的命令"
                        }
                    }
                }
            }
        }
    }
}
