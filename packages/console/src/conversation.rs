//! 对话状态机
//!
//! 管理多轮对话的生命周期：Idle → Parsing → Confirming → Executing → Done
//! 支持 LLM 驱动的意图解析（优先），本地规则 fallback（离线可用）。

use crate::intent::{self, IntentResult, IntentError, ParsedIntent};
use dioxus::prelude::*;

/// 对话状态
#[derive(Clone, Debug, PartialEq)]
pub enum ConversationState {
    /// 空闲状态，等待用户输入
    Idle,
    /// 正在解析中（LLM 请求进行中）
    Parsing {
        /// 对话历史
        history: Vec<ChatMessage>,
    },
    /// 已解析意图，等待用户确认
    Confirming {
        /// 解析结果
        result: IntentResult,
        /// 对话历史（用于展示）
        history: Vec<ChatMessage>,
    },
    /// 参数不完整，需要追问
    Asking {
        /// 部分解析结果
        partial: IntentResult,
        /// 追问提示
        prompt: String,
        /// 对话历史
        history: Vec<ChatMessage>,
    },
    /// 正在执行
    Executing {
        /// 执行中的意图
        intent: ParsedIntent,
    },
    /// 执行完成
    Done {
        /// 操作描述
        summary: String,
        /// 是否成功
        success: bool,
    },
    /// 解析错误
    Error {
        /// 错误信息
        message: String,
    },
}

/// 对话消息
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    /// 消息来源
    pub role: MessageRole,
    /// 消息内容
    pub content: String,
    /// 时间戳
    pub timestamp: String,
}

/// 消息来源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageRole {
    User,
    System,
}

/// 操作历史记录
#[derive(Clone, Debug, PartialEq)]
pub struct OperationRecord {
    pub description: String,
    pub intent: ParsedIntent,
    pub timestamp: String,
    pub undoable: bool,
}

/// 对话管理器
#[derive(Clone, Debug, PartialEq)]
pub struct ConversationManager {
    /// 当前状态
    pub state: ConversationState,
    /// 操作历史
    pub operation_history: Vec<OperationRecord>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            state: ConversationState::Idle,
            operation_history: Vec::new(),
        }
    }

    /// 同步处理用户输入（本地 fallback）
    pub fn process_input(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return;
        }

        match &self.state {
            ConversationState::Idle => {
                self.handle_new_input_local(trimmed);
            }
            ConversationState::Asking { partial, prompt: _, history } => {
                self.handle_supplementary_input(trimmed, partial.clone(), history.clone());
            }
            ConversationState::Confirming { .. } => {
                if trimmed == "确认" || trimmed == "确认执行" || trimmed == "执行" || trimmed == "是" || trimmed == "好的" {
                    self.execute_current();
                } else if trimmed == "取消" || trimmed == "算了" || trimmed == "不" {
                    self.state = ConversationState::Idle;
                } else {
                    self.handle_new_input_local(trimmed);
                }
            }
            ConversationState::Done { .. } | ConversationState::Error { .. } => {
                self.handle_new_input_local(trimmed);
            }
            ConversationState::Parsing { .. } | ConversationState::Executing { .. } => {}
        }
    }

    /// 开始异步 LLM 解析：设置 Parsing 状态并返回需要的信息
    /// 调用方在释放写锁后再发起异步请求
    pub fn start_parsing(&mut self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        match &self.state {
            ConversationState::Idle | ConversationState::Done { .. } | ConversationState::Error { .. } => {
                let history = vec![ChatMessage {
                    role: MessageRole::User,
                    content: trimmed.to_string(),
                    timestamp: now_timestamp(),
                }];
                self.state = ConversationState::Parsing { history };
                true
            }
            ConversationState::Asking { partial: _, prompt: _, history } => {
                let mut new_history = history.clone();
                new_history.push(ChatMessage {
                    role: MessageRole::User,
                    content: trimmed.to_string(),
                    timestamp: now_timestamp(),
                });
                // 保存 partial 到 history 的最后以便后续恢复
                self.state = ConversationState::Parsing { history: new_history };
                true
            }
            ConversationState::Confirming { .. } => {
                if trimmed == "确认" || trimmed == "确认执行" || trimmed == "执行" || trimmed == "是" || trimmed == "好的" {
                    self.execute_current();
                } else if trimmed == "取消" || trimmed == "算了" || trimmed == "不" {
                    self.state = ConversationState::Idle;
                } else {
                    let history = vec![ChatMessage {
                        role: MessageRole::User,
                        content: trimmed.to_string(),
                        timestamp: now_timestamp(),
                    }];
                    self.state = ConversationState::Parsing { history };
                    return true;
                }
                false
            }
            ConversationState::Parsing { .. } | ConversationState::Executing { .. } => false,
        }
    }

    /// 应用 LLM 异步解析结果（从外部调用，不持有写锁跨 await）
    pub fn apply_llm_result(&mut self, user_input: &str, result: Result<IntentResult, IntentError>) {
        // 从 Parsing 状态中取出 history，或新建
        let history = match &self.state {
            ConversationState::Parsing { history } => history.clone(),
            _ => Vec::new(),
        };

        let mut history = history;
        // 避免重复添加用户消息（start_parsing 已添加）
        // history 中最后一条已经是用户消息

        match result {
            Ok(intent_result) => {
                self.apply_intent_result(intent_result, history);
            }
            Err(e) => {
                let error_msg = format!("⚠️ AI 解析失败，原因: {e}");
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: error_msg.clone(),
                    timestamp: now_timestamp(),
                });
                // Fallback 到本地解析
                if let Some(local_result) = intent::parse_intent(user_input) {
                    history.push(ChatMessage {
                        role: MessageRole::System,
                        content: "已切换到本地模式解析。".to_string(),
                        timestamp: now_timestamp(),
                    });
                    self.apply_intent_result(local_result, history);
                } else {
                    self.state = ConversationState::Error { message: error_msg };
                }
            }
        }
    }

    /// 应用解析结果到状态机
    fn apply_intent_result(&mut self, result: IntentResult, mut history: Vec<ChatMessage>) {
        match &result.intent {
            ParsedIntent::Cancel => {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: "已取消当前操作。".to_string(),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Done {
                    summary: "已取消".to_string(),
                    success: true,
                };
            }
            ParsedIntent::Search { .. } => {
                let desc = result.intent.to_string();
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: format!("✅ {desc}"),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Done {
                    summary: desc,
                    success: true,
                };
            }
            ParsedIntent::Unknown { message: msg } => {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: format!("🤔 不太理解你的意思「{msg}」，请换种方式描述？"),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Idle;
            }
            _ => {
                if result.missing_fields.is_empty() {
                    let hint = result.hint.clone();
                    history.push(ChatMessage {
                        role: MessageRole::System,
                        content: format!("{}\n{}", hint, result.intent),
                        timestamp: now_timestamp(),
                    });
                    history.push(ChatMessage {
                        role: MessageRole::System,
                        content: "请确认是否执行？".to_string(),
                        timestamp: now_timestamp(),
                    });
                    self.state = ConversationState::Confirming { result, history };
                } else {
                    let hint = result.hint.clone();
                    if let Some(ask_text) = result.intent.ask_for_missing() {
                        history.push(ChatMessage {
                            role: MessageRole::System,
                            content: format!("{hint}\n{ask_text}"),
                            timestamp: now_timestamp(),
                        });
                        self.state = ConversationState::Asking {
                            partial: result,
                            prompt: ask_text,
                            history,
                        };
                    } else {
                        self.state = ConversationState::Idle;
                    }
                }
            }
        }
    }

    // ---- 本地同步处理 ----

    fn handle_new_input_local(&mut self, input: &str) {
        if let Some(result) = intent::parse_intent(input) {
            let history = vec![ChatMessage {
                role: MessageRole::User,
                content: input.to_string(),
                timestamp: now_timestamp(),
            }];
            self.apply_intent_result(result, history);
        }
    }

    fn handle_supplementary_input(
        &mut self,
        input: &str,
        partial: IntentResult,
        mut history: Vec<ChatMessage>,
    ) {
        history.push(ChatMessage {
            role: MessageRole::User,
            content: input.to_string(),
            timestamp: now_timestamp(),
        });

        if let Some(new_result) = intent::parse_intent(input) {
            let merged = self.merge_intents(&partial.intent, &new_result.intent);
            if merged.missing_fields().is_empty() {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: format!("{}\n{}", merged, "请确认是否执行？"),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Confirming {
                    result: IntentResult {
                        intent: merged,
                        confidence: new_result.confidence.max(partial.confidence),
                        hint: new_result.hint,
                        missing_fields: Vec::new(),
                    },
                    history,
                };
            } else if let Some(ask_text) = merged.ask_for_missing() {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: ask_text.clone(),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Asking {
                    partial: IntentResult {
                        intent: merged,
                        confidence: new_result.confidence.max(partial.confidence),
                        hint: new_result.hint,
                        missing_fields: new_result.missing_fields,
                    },
                    prompt: ask_text,
                    history,
                };
            }
        } else {
            let updated = self.fill_missing_from_text(&partial.intent, input);
            if updated.missing_fields().is_empty() {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: format!("{}\n{}", updated, "请确认是否执行？"),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Confirming {
                    result: IntentResult {
                        intent: updated,
                        confidence: partial.confidence + 0.1,
                        hint: partial.hint,
                        missing_fields: Vec::new(),
                    },
                    history,
                };
            } else if let Some(ask_text) = updated.ask_for_missing() {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: ask_text.clone(),
                    timestamp: now_timestamp(),
                });
                self.state = ConversationState::Asking {
                    partial: IntentResult {
                        intent: updated,
                        confidence: partial.confidence,
                        hint: partial.hint,
                        missing_fields: partial.missing_fields,
                    },
                    prompt: ask_text,
                    history,
                };
            }
        }
    }

    /// 合并两个意图（取已知参数）
    fn merge_intents(&self, old: &ParsedIntent, new: &ParsedIntent) -> ParsedIntent {
        match (old, new) {
            (
                ParsedIntent::Transfer { employee_name: old_emp, from_department: old_from, to_department: old_to },
                ParsedIntent::Transfer { employee_name: new_emp, from_department: new_from, to_department: new_to },
            ) => ParsedIntent::Transfer {
                employee_name: new_emp.clone().or_else(|| old_emp.clone()),
                from_department: new_from.clone().or_else(|| old_from.clone()),
                to_department: new_to.clone().or_else(|| old_to.clone()),
            },
            (
                ParsedIntent::CreateDepartment { department_name: old_name, parent_department: old_parent },
                ParsedIntent::CreateDepartment { department_name: new_name, parent_department: new_parent },
            ) => ParsedIntent::CreateDepartment {
                department_name: new_name.clone().or_else(|| old_name.clone()),
                parent_department: new_parent.clone().or_else(|| old_parent.clone()),
            },
            (_, new_intent) => new_intent.clone(),
        }
    }

    /// 从文本中填充缺失参数
    fn fill_missing_from_text(&self, intent: &ParsedIntent, text: &str) -> ParsedIntent {
        match intent {
            ParsedIntent::Transfer { employee_name, from_department, to_department } => {
                let emp = if employee_name.is_none() { Some(text.to_string()) } else { employee_name.clone() };
                let to = if to_department.is_none() && employee_name.is_some() { Some(text.to_string()) } else { to_department.clone() };
                ParsedIntent::Transfer { employee_name: emp, from_department: from_department.clone(), to_department: to }
            }
            ParsedIntent::CreateDepartment { department_name, parent_department } => {
                let name = if department_name.is_none() { Some(text.to_string()) } else { department_name.clone() };
                ParsedIntent::CreateDepartment { department_name: name, parent_department: parent_department.clone() }
            }
            other => other.clone(),
        }
    }

    /// 执行当前确认的操作 → 进入 Executing 状态
    /// 实际 API 调用由 UI 层异步完成
    pub fn execute_current(&mut self) {
        if let ConversationState::Confirming { result, history, .. } = &self.state {
            let intent = result.intent.clone();
            let desc = intent.to_string();
            let mut history = history.clone();

            history.push(ChatMessage {
                role: MessageRole::System,
                content: "⏳ 正在执行...".to_string(),
                timestamp: now_timestamp(),
            });

            self.operation_history.push(OperationRecord {
                description: desc,
                intent: intent.clone(),
                timestamp: now_timestamp(),
                undoable: true,
            });

            self.state = ConversationState::Executing { intent };
        }
    }

    /// 应用执行结果
    pub fn apply_execution_result(&mut self, success: bool, summary: String, detail: Option<String>) {
        let mut msg = summary.clone();
        if let Some(d) = detail {
            msg = format!("{msg}\n{d}");
        }
        self.state = ConversationState::Done {
            summary: msg,
            success,
        };
    }

    pub fn confirm(&mut self) { self.execute_current(); }
    pub fn cancel(&mut self) { self.state = ConversationState::Idle; }
    pub fn reset(&mut self) { self.state = ConversationState::Idle; }

    pub fn history(&self) -> &[ChatMessage] {
        match &self.state {
            ConversationState::Confirming { history, .. } => history,
            ConversationState::Asking { history, .. } => history,
            ConversationState::Parsing { history } => history,
            ConversationState::Error { .. } => &[],
            _ => &[],
        }
    }

    pub fn current_intent(&self) -> Option<&ParsedIntent> {
        match &self.state {
            ConversationState::Confirming { result, .. } => Some(&result.intent),
            ConversationState::Asking { partial, .. } => Some(&partial.intent),
            ConversationState::Executing { intent } => Some(intent),
            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, ConversationState::Idle)
    }

    pub fn is_parsing(&self) -> bool {
        matches!(self.state, ConversationState::Parsing { .. })
    }
}

fn now_timestamp() -> String {
    "刚刚".to_string()
}