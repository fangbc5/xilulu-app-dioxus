//! ⌘K 命令面板组件
//!
//! 全局快捷键触发的命令搜索面板，聚合所有模块注册的命令。
//! 支持模糊搜索、键盘导航、分组展示。
//! Phase 5: LLM 驱动的意图解析，本地规则 fallback。

use crate::conversation::{ConversationManager, ConversationState, MessageRole};
use crate::executor;
use crate::intent::{self, LlmConfig};
use crate::module::{Command, CommandAction, CommandCategory};
use crate::services::client::ApiClient;
use crate::skill::SkillStore;
use dioxus::prelude::*;

/// 命令面板全局状态
#[derive(Clone, Copy)]
pub struct CommandPaletteState {
    pub is_open: Signal<bool>,
}

/// 命令面板 Provider
#[component]
pub fn CommandPaletteProvider(
    commands: Vec<Command>,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| false);

    use_context_provider(|| CommandPaletteState { is_open });

    // 通过 JS 注册 document 级键盘事件
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
    let mut conversation = use_signal(ConversationManager::new);
    let mut skill_store = use_signal(SkillStore::new);
    let nav = use_navigator();

    // LLM 配置（从 localStorage 或默认值）
    let llm_config = use_signal(LlmConfig::default);

    // 共享 API 客户端（携带当前 access_token）
    let api_client = use_context::<ApiClient>();

    // 检测 Executing 状态 → 异步调用 API
    let is_executing = matches!(conversation.read().state, ConversationState::Executing { .. });
    use_effect(move || {
        if !is_executing {
            return;
        }
        let intent = match &conversation.read().state {
            ConversationState::Executing { intent } => intent.clone(),
            _ => return,
        };
        let client = api_client.clone();
        spawn(async move {
            let result = executor::execute_intent(&client, &intent, 1).await;
            conversation.write().apply_execution_result(
                result.success,
                result.summary,
                result.detail,
            );
            if result.success {
                skill_store.write().record_success(&intent);
            }
        });
    });

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
    let is_conversing = conversation.read().is_active();
    let is_parsing = conversation.read().is_parsing();

    rsx! {
        div {
            class: "command-overlay",
            onclick: move |_| {
                conversation.write().reset();
                on_close.call(());
            },
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    conversation.write().reset();
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
                    placeholder: if is_parsing { "🧠 AI 正在思考..." } else if is_conversing { "继续输入或输入「确认」/「取消」..." } else { "输入命令、搜索、或描述你想做的事..." },
                    autofocus: true,
                    disabled: is_parsing,
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
                                let input_val = query();
                                if is_parsing {
                                    return;
                                }
                                if input_val.is_empty() {
                                    // 空输入：选择当前高亮命令
                                    if let Some(cmd) = filtered.get(selected_index()) {
                                        match &cmd.action {
                                            CommandAction::Navigate(path) => {
                                                nav.push(path.as_str());
                                            }
                                            CommandAction::OpenPanel(_) => {}
                                        }
                                        on_close.call(());
                                    }
                                } else {
                                    // 有输入：先设置 Parsing 状态（短暂获取写锁）
                                    let should_parse = conversation.write().start_parsing(&input_val);
                                    if should_parse {
                                        // 释放写锁后再发起异步请求
                                        let config = llm_config.read().clone();
                                        let input_for_async = input_val.clone();
                                        spawn(async move {
                                            let result = intent::parse_intent_llm(
                                                    &input_for_async,
                                                    &[],
                                                    &config,
                                                )
                                                .await;
                                            conversation
                                                .write()
                                                .apply_llm_result(&input_for_async, result);
                                        });
                                        query.set(String::new());
                                    }
                                }
                            }
                            Key::Escape => {
                                conversation.write().reset();
                                on_close.call(());
                            }
                            _ => {}
                        }
                    },
                }

                // 对话内容区域
                div { class: "command-list",

                    // === 对话模式 ===
                    if is_conversing {
                        {render_conversation(conversation)}

                        // 确认/取消按钮
                        {render_conversation_actions(conversation)}
                    } else if !query().is_empty() {
                        // 意图解析提示（本地即时预览）
                        {render_intent_hint(&query())}

                        // 导航分组
                        if !nav_cmds.is_empty() {
                            div { class: "command-group-label", "导航" }
                            for (i , cmd) in nav_cmds.iter().enumerate() {
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
                            for (i , cmd) in action_cmds.iter().enumerate() {
                                {
                                    let global_i = nav_count + i;
                                    let is_sel = global_i == selected_index();
                                    let sel_class = if is_sel { " selected" } else { "" };
                                    let name = cmd.name.clone();
                                    let desc = cmd.description.clone();
                                    rsx! {
                                        div { class: "command-item{sel_class}",
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
                            div { style: "padding: 24px; text-align: center; color: var(--ds-text-tertiary); font-size: 14px;",
                                "没有匹配的命令，按 Enter 让 AI 帮你理解"
                            }
                        }
                    } else {
                        div { class: "command-group-label", "💡 试试说" }
                        for suggestion in [
                            ("把张三从市场部调到产品部", "🔄 员工调岗"),
                            ("新建部门 AI实验室", "📂 创建部门"),
                            ("搜索张三", "🔍 搜索"),
                        ]
                        {
                            {
                                let (full_text, label) = suggestion;
                                let full_text_for_click = full_text.to_string();
                                rsx! {
                                    div {
                                        class: "command-item",
                                        style: "cursor: pointer;",
                                        onclick: move |_| {
                                            query.set(full_text_for_click.clone());
                                        },
                                        span { class: "cmd-icon", "💡" }
                                        span { class: "cmd-name", "{label}" }
                                        span { class: "cmd-desc", "{full_text}" }
                                    }
                                }
                            }
                        }

                        div { class: "command-group-label", "快捷命令" }
                        for cmd in commands.iter().take(5) {
                            {
                                let name = cmd.name.clone();
                                let desc = cmd.description.clone();
                                let action = cmd.action.clone();
                                rsx! {
                                    div {
                                        class: "command-item",
                                        style: "cursor: pointer;",
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
                }
            }
        }
    }
}

/// 渲染意图解析提示（本地即时预览）
fn render_intent_hint(query: &str) -> Element {
    if let Some(intent_result) = intent::parse_intent(query) {
        let missing_count = intent_result.missing_fields.len();
        rsx! {
            div { style: "padding: 8px 16px; border-bottom: 1px solid var(--ds-border);",

                div { style: "font-size: 12px; color: var(--ds-accent); margin-bottom: 4px;",
                    "{intent_result.hint}"
                }
                div { style: "font-size: 13px; color: var(--ds-text-primary);",
                    "{intent_result.intent}"
                }
                if missing_count > 0 {
                    div { style: "font-size: 11px; color: var(--ds-text-tertiary); margin-top: 4px;",
                        "按 Enter 继续（{missing_count} 个参数待补充，AI 将辅助补全）"
                    }
                } else {
                    div { style: "font-size: 11px; color: var(--ds-text-tertiary); margin-top: 4px;",
                        "按 Enter 确认执行"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

/// 渲染对话历史
fn render_conversation(conversation: Signal<ConversationManager>) -> Element {
    let history = conversation.read().history().to_vec();

    rsx! {
        for msg in history.iter() {
            {
                let is_user = msg.role == MessageRole::User;
                let bg = if is_user {
                    "background: var(--ds-accent); color: var(--ds-text-inverse);".to_string()
                } else {
                    "background: var(--ds-bg-elevated); color: var(--ds-text-primary); border: 1px solid var(--ds-border);"
                        .to_string()
                };
                let align = if is_user { "margin-left: auto;" } else { "margin-right: auto;" };
                let content = msg.content.clone();
                rsx! {
                    div { style: "max-width: 85%; padding: 8px 12px; border-radius: 10px; font-size: 13px; line-height: 1.5; margin: 4px 16px; {bg} {align}",
                        "{content}"
                    }
                }
            }
        }
    }
}

/// 渲染对话操作按钮
fn render_conversation_actions(mut conversation: Signal<ConversationManager>) -> Element {
    match &conversation.read().state {
        ConversationState::Confirming { result, .. } => {
            let intent_display = result.intent.to_string();
            rsx! {
                div { style: "display: flex; gap: 8px; padding: 8px 16px;",

                    button {
                        style: "flex: 1; padding: 8px 16px; border-radius: 8px; border: 1px solid var(--ds-accent); background: var(--ds-accent); color: var(--ds-text-inverse); font-size: 13px; font-weight: 600; cursor: pointer;",
                        onclick: move |_| {
                            conversation.write().confirm();
                        },
                        "✓ 确认执行"
                    }
                    button {
                        style: "flex: 1; padding: 8px 16px; border-radius: 8px; border: 1px solid var(--ds-border); background: transparent; color: var(--ds-text-secondary); font-size: 13px; cursor: pointer;",
                        onclick: move |_| {
                            conversation.write().cancel();
                        },
                        "✕ 取消"
                    }
                }
                div { style: "padding: 4px 16px 8px; font-size: 11px; color: var(--ds-text-tertiary); text-align: center;",
                    "{intent_display}"
                }
            }
        }
        ConversationState::Asking { prompt, .. } => {
            let prompt_text = prompt.clone();
            rsx! {
                div { style: "padding: 12px 16px; color: var(--ds-accent); font-size: 13px; border-top: 1px solid var(--ds-border);",
                    "💬 {prompt_text}"
                }
            }
        }
        ConversationState::Done { summary, .. } => {
            let summary_text = summary.clone();
            rsx! {
                div { style: "padding: 12px 16px; text-align: center; color: var(--ds-text-success, #4ade80); font-size: 13px; border-top: 1px solid var(--ds-border);",
                    "{summary_text}"
                }
            }
        }
        ConversationState::Executing { .. } => {
            rsx! {
                div { style: "padding: 12px 16px; text-align: center; color: var(--ds-accent); font-size: 13px; border-top: 1px solid var(--ds-border);",
                    "⏳ 正在调用 API 执行..."
                }
            }
        }
        ConversationState::Parsing { .. } => {
            rsx! {
                div { style: "padding: 12px 16px; text-align: center; color: var(--ds-accent); font-size: 13px; border-top: 1px solid var(--ds-border);",
                    "🧠 AI 正在理解你的意图..."
                }
            }
        }
        ConversationState::Error { message } => {
            let msg = message.clone();
            rsx! {
                div { style: "padding: 12px 16px; text-align: center; color: #f87171; font-size: 13px; border-top: 1px solid var(--ds-border);",
                    "{msg}"
                }
            }
        }
        _ => rsx! {},
    }
}