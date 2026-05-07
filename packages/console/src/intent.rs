//! 自然语言意图解析引擎
//!
//! 支持两种模式：
//! 1. LLM 驱动（推荐）：调用 OpenAI 兼容 API，返回结构化意图 JSON
//! 2. 本地规则（离线 fallback）：关键词 + 字符串匹配

use std::fmt;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

// ============================================================
// 数据类型
// ============================================================

/// 解析后的意图
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedIntent {
    /// 员工调岗意图
    Transfer {
        /// 员工姓名（可能未提取到）
        employee_name: Option<String>,
        /// 原部门（可能未提取到）
        from_department: Option<String>,
        /// 目标部门（可能未提取到）
        to_department: Option<String>,
    },
    /// 新建部门意图
    CreateDepartment {
        /// 部门名称（可能未提取到）
        department_name: Option<String>,
        /// 上级部门（可能未提取到）
        parent_department: Option<String>,
    },
    /// 搜索意图
    Search {
        /// 搜索关键词
        keyword: String,
        /// 搜索类型：employee / department / all
        scope: SearchScope,
    },
    /// 取消当前操作
    Cancel,
    /// 无法识别的输入
    Unknown { message: String },
}

/// 搜索范围
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    Employee,
    Department,
    All,
}

/// 意图解析结果
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntentResult {
    /// 解析出的意图
    pub intent: ParsedIntent,
    /// 置信度 (0.0 ~ 1.0)
    pub confidence: f32,
    /// 解析提示（用于展示给用户）
    pub hint: String,
    /// 缺失的参数字段名
    pub missing_fields: Vec<String>,
}

/// LLM API 配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API base URL，例如 "http://localhost:11434/v1" (Ollama) 或 "https://api.openai.com/v1"
    pub api_base: String,
    /// API Key（本地模型可留空）
    pub api_key: Option<String>,
    /// 模型名称，例如 "gpt-4o-mini" 或 "qwen2.5:7b"
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_base: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: "qwen2:1.5b-instruct".to_string(),
        }
    }
}

// ============================================================
// LLM 驱动的意图解析
// ============================================================

/// 系统提示词：指导 LLM 返回结构化 JSON
const INTENT_SYSTEM_PROMPT: &str = r#"你是一个企业组织管理系统的意图解析引擎。你的任务是将用户的自然语言输入解析为结构化意图。

支持的操作类型：
1. **员工调岗** (transfer)：将员工从一个部门调到另一个部门
2. **新建部门** (create_department)：创建新的部门/团队
3. **搜索** (search)：搜索员工、部门信息
4. **取消** (cancel)：取消当前操作
5. **未知** (unknown)：无法识别的输入

你必须返回严格的 JSON 格式（不要 markdown 代码块），格式如下：

调岗：
{"type":"transfer","employee_name":"张三","from_department":"市场部","to_department":"产品部"}
缺少信息时对应字段设为 null。

新建部门：
{"type":"create_department","department_name":"AI实验室","parent_department":"技术中心"}
缺少信息时对应字段设为 null。

搜索：
{"type":"search","keyword":"张三","scope":"employee"}
scope 可选值：employee, department, all

取消：
{"type":"cancel"}

无法识别：
{"type":"unknown","message":"用户原始输入"}

只返回 JSON，不要任何其他文字。如果用户提供了多轮对话上下文，合并之前的信息来补全缺失字段。"#;

/// 异步 LLM 意图解析请求
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// LLM 响应结构
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

/// 使用 LLM 解析用户意图（异步）
pub async fn parse_intent_llm(
    user_input: &str,
    conversation_history: &[(&str, &str)], // (role, content) pairs
    config: &LlmConfig,
) -> Result<IntentResult, IntentError> {
    // 构建消息列表
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: INTENT_SYSTEM_PROMPT.to_string(),
    }];

    // 添加对话历史
    for (role, content) in conversation_history {
        messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    // 添加当前用户输入
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_input.to_string(),
    });

    let request_body = ChatRequest {
        model: config.model.clone(),
        messages,
        temperature: 0.1, // 低温度，确保稳定输出
    };

    let url = format!("{}/chat/completions", config.api_base.trim_end_matches('/'));
    let json_body = serde_json::to_string(&request_body)
        .map_err(|e| IntentError::Serialization(e.to_string()))?;

    let mut builder = Request::post(&url)
        .header("Content-Type", "application/json");

    if let Some(ref key) = config.api_key {
        builder = builder.header("Authorization", &format!("Bearer {key}"));
    }

    let resp = builder
        .body(json_body)
        .map_err(|e| IntentError::Network(e.to_string()))?
        .send()
        .await
        .map_err(|e| IntentError::Network(e.to_string()))?;
    let status = resp.status();

    if !(200..300).contains(&status) {
        let text = resp.text().await.unwrap_or_default();
        return Err(IntentError::LlmApi(format!("HTTP {status}: {text}")));
    }

    let text = resp.text().await.map_err(|e| IntentError::Network(e.to_string()))?;
    let chat_resp: ChatResponse = serde_json::from_str(&text)
        .map_err(|e| IntentError::Serialization(format!("Failed to parse LLM response: {e}")))?;

    let content = chat_resp.choices.first()
        .ok_or(IntentError::LlmApi("No choices in response".to_string()))?
        .message.content.clone();

    // 解析 LLM 返回的 JSON
    parse_llm_json(&content)
}

/// 解析 LLM 返回的 JSON 字符串为 IntentResult
fn parse_llm_json(json_str: &str) -> Result<IntentResult, IntentError> {
    // 清理可能的 markdown 代码块标记
    let cleaned = json_str
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let intent: ParsedIntent = serde_json::from_str(cleaned)
        .map_err(|e| IntentError::Serialization(format!("Invalid intent JSON: {e}\nRaw: {cleaned}")))?;

    let missing = intent.missing_fields();
    let confidence = if missing.is_empty() { 0.95 } else { 0.75 };
    let hint = format!("🧠 识别意图: {}", intent.intent_type_name());

    Ok(IntentResult {
        intent,
        confidence,
        hint,
        missing_fields: missing,
    })
}

/// 意图解析错误
#[derive(Debug)]
pub enum IntentError {
    /// 网络错误
    Network(String),
    /// 序列化/反序列化错误
    Serialization(String),
    /// LLM API 返回错误
    LlmApi(String),
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::Network(e) => write!(f, "网络错误: {e}"),
            IntentError::Serialization(e) => write!(f, "解析错误: {e}"),
            IntentError::LlmApi(e) => write!(f, "AI 服务错误: {e}"),
        }
    }
}

impl std::error::Error for IntentError {}

// ============================================================
// 本地规则 fallback（离线可用）
// ============================================================

/// 从用户输入解析意图（本地同步 fallback）
pub fn parse_intent(input: &str) -> Option<IntentResult> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 按优先级尝试各种意图匹配
    if let Some(result) = try_parse_cancel(trimmed) {
        return Some(result);
    }
    if let Some(result) = try_parse_transfer(trimmed) {
        return Some(result);
    }
    if let Some(result) = try_parse_create_department(trimmed) {
        return Some(result);
    }
    if let Some(result) = try_parse_search(trimmed) {
        return Some(result);
    }

    None
}

/// 解析调岗意图
fn try_parse_transfer(input: &str) -> Option<IntentResult> {
    let transfer_keywords = ["调到", "调去", "调岗", "转到", "转去", "转岗"];
    let has_transfer_intent = transfer_keywords.iter().any(|kw| input.contains(kw))
        || (input.contains("从") && input.contains("到") && !input.contains("新建"));

    if !has_transfer_intent {
        return None;
    }

    let mut employee_name = None;
    let mut from_department = None;
    let mut to_department = None;

    let stripped = input
        .strip_prefix("把")
        .or_else(|| input.strip_prefix("将"))
        .unwrap_or(input);

    if let Some(from_pos) = stripped.find("从") {
        if let Some(transfer_pos) = find_transfer_to(stripped) {
            let name_part = stripped[..from_pos].trim();
            let from_part = stripped[from_pos + "从".len()..transfer_pos.0].trim();
            let to_part = stripped[transfer_pos.1..].trim();

            if !name_part.is_empty() { employee_name = Some(name_part.to_string()); }
            if !from_part.is_empty() { from_department = Some(from_part.to_string()); }
            if !to_part.is_empty() { to_department = Some(to_part.to_string()); }
        }
    } else if let Some(transfer_pos) = find_transfer_to(stripped) {
        let name_part = stripped[..transfer_pos.0].trim();
        let to_part = stripped[transfer_pos.1..].trim();
        if !name_part.is_empty() { employee_name = Some(name_part.to_string()); }
        if !to_part.is_empty() { to_department = Some(to_part.to_string()); }
    } else if let Some(pos) = stripped.find("调岗") {
        let name_part = stripped[..pos].trim();
        let rest = stripped[pos + "调岗".len()..]
            .trim_start_matches("到")
            .trim_start_matches("去")
            .trim();
        if !name_part.is_empty() { employee_name = Some(name_part.to_string()); }
        if !rest.is_empty() { to_department = Some(rest.to_string()); }
    }

    if employee_name.is_none() && to_department.is_none() {
        return None;
    }

    let intent = ParsedIntent::Transfer { employee_name, from_department, to_department };
    let missing = intent.missing_fields();
    let confidence = if missing.is_empty() { 0.92 } else { 0.7 };

    Some(IntentResult {
        hint: format!("🧠 识别意图: {}", intent.intent_type_name()),
        confidence,
        intent,
        missing_fields: missing,
    })
}

fn find_transfer_to(s: &str) -> Option<(usize, usize)> {
    for kw in &["调到", "转到", "调去", "转去"] {
        if let Some(pos) = s.find(kw) {
            return Some((pos, pos + kw.len()));
        }
    }
    None
}

fn try_parse_create_department(input: &str) -> Option<IntentResult> {
    let create_keywords = ["新建部门", "创建部门", "新建一个部门", "创建一个部门", "添加部门", "添加一个部门"];
    let has_create_intent = create_keywords.iter().any(|kw| input.contains(kw));

    if !has_create_intent {
        if !input.contains("新建") && !input.contains("创建") {
            return None;
        }
    }

    let mut department_name = None;
    let mut parent_department = None;

    // "在 Y 下面/下 新建/创建部门 X"
    if let Some(at_pos) = input.find("在") {
        if let Some(below_pos) = input.find("下面").or_else(|| input.find("下")) {
            if below_pos > at_pos {
                let after_below = &input[below_pos..];
                if after_below.contains("新建") || after_below.contains("创建") {
                    parent_department = Some(input[at_pos + "在".len()..below_pos].trim().to_string());
                    if let Some(dept_pos) = after_below.find("部门") {
                        let after_dept = after_below[dept_pos + "部门".len()..].to_string();
                        let name = after_dept
                            .trim_start_matches(|c: char| c == '叫' || c == '：' || c == ':' || c == ' ')
                            .trim();
                        if !name.is_empty() {
                            department_name = Some(name.to_string());
                        }
                    }
                }
            }
        }
    }

    // "新建/创建部门 X [在/放在 Y 下面]"
    if department_name.is_none() {
        let dept_prefix = find_create_dept_prefix(input);
        if let Some((_start, end)) = dept_prefix {
            let after = input[end..].to_string();
            let name_and_maybe_parent = after
                .trim_start_matches(|c: char| c == '叫' || c == '：' || c == ':' || c == ' ')
                .trim();

            if let Some(in_pos) = name_and_maybe_parent.find(" 放在").or_else(|| name_and_maybe_parent.find(" 在")) {
                let name_part = name_and_maybe_parent[..in_pos].trim();
                let rest = &name_and_maybe_parent[in_pos..]
                    .trim_start_matches(" 放在")
                    .trim_start_matches(" 在")
                    .trim();
                department_name = Some(name_part.to_string());
                parent_department = Some(rest.trim_end_matches("下面").trim_end_matches("下").trim().to_string());
            } else {
                let name = name_and_maybe_parent.trim();
                if !name.is_empty() {
                    department_name = Some(name.to_string());
                }
            }
        }
    }

    if department_name.is_none() {
        return None;
    }

    let intent = ParsedIntent::CreateDepartment { department_name, parent_department };
    let missing = intent.missing_fields();
    let confidence = if missing.is_empty() { 0.9 } else { 0.75 };

    Some(IntentResult {
        hint: format!("🧠 识别意图: {}", intent.intent_type_name()),
        confidence,
        intent,
        missing_fields: missing,
    })
}

fn find_create_dept_prefix(input: &str) -> Option<(usize, usize)> {
    for prefix in &["新建一个部门", "创建一个部门", "新建部门", "创建部门", "添加一个部门", "添加部门"] {
        if let Some(pos) = input.find(prefix) {
            let end = pos + prefix.len();
            return Some((pos, end));
        }
    }
    None
}

fn try_parse_search(input: &str) -> Option<IntentResult> {
    let search_keywords = ["搜索", "查找", "查找一下", "找一下", "搜", "找"];
    let has_search_intent = search_keywords.iter().any(|kw| input.starts_with(kw));

    if !has_search_intent {
        return None;
    }

    let keyword = input
        .trim_start_matches("搜索")
        .trim_start_matches("查找一下")
        .trim_start_matches("查找")
        .trim_start_matches("找一下")
        .trim_start_matches("搜")
        .trim_start_matches("找")
        .trim();

    if keyword.is_empty() {
        return None;
    }

    let scope = if keyword.contains("部") || keyword.contains("部门") {
        SearchScope::Department
    } else if keyword.contains("员工") || keyword.contains("人") {
        SearchScope::Employee
    } else {
        SearchScope::All
    };

    let clean_keyword = keyword
        .replace("员工", "")
        .replace("部门", "")
        .replace("的", "")
        .replace("人员", "")
        .replace("成员", "")
        .trim()
        .to_string();

    if clean_keyword.is_empty() {
        return None;
    }

    Some(IntentResult {
        intent: ParsedIntent::Search { keyword: clean_keyword.clone(), scope },
        hint: format!("🧠 识别意图: 搜索「{clean_keyword}」"),
        confidence: 0.85,
        missing_fields: Vec::new(),
    })
}

fn try_parse_cancel(input: &str) -> Option<IntentResult> {
    let cancel_keywords = ["取消", "算了", "不要了", "放弃", "取消操作", "不执行"];
    if cancel_keywords.iter().any(|kw| input == *kw || input.starts_with(kw)) {
        Some(IntentResult {
            intent: ParsedIntent::Cancel,
            hint: "🧠 识别意图: 取消操作".to_string(),
            confidence: 0.95,
            missing_fields: Vec::new(),
        })
    } else {
        None
    }
}

// ============================================================
// 共享方法
// ============================================================

impl ParsedIntent {
    pub fn intent_type_name(&self) -> &'static str {
        match self {
            Self::Transfer { .. } => "员工调岗",
            Self::CreateDepartment { .. } => "新建部门",
            Self::Search { .. } => "搜索",
            Self::Cancel => "取消操作",
            Self::Unknown { .. } => "未识别",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Transfer { .. } => "🔄",
            Self::CreateDepartment { .. } => "📂",
            Self::Search { .. } => "🔍",
            Self::Cancel => "❌",
            Self::Unknown { .. } => "❓",
        }
    }

    pub fn is_complete(&self) -> bool {
        self.missing_fields().is_empty()
    }

    pub fn missing_fields(&self) -> Vec<String> {
        match self {
            Self::Transfer { employee_name, from_department, to_department } => {
                let mut missing = Vec::new();
                if employee_name.is_none() { missing.push("员工姓名".to_string()); }
                if to_department.is_none() { missing.push("目标部门".to_string()); }
                let _ = from_department;
                missing
            }
            Self::CreateDepartment { department_name, parent_department } => {
                let mut missing = Vec::new();
                if department_name.is_none() { missing.push("部门名称".to_string()); }
                let _ = parent_department;
                missing
            }
            Self::Search { .. } | Self::Cancel | Self::Unknown { .. } => Vec::new(),
        }
    }

    pub fn ask_for_missing(&self) -> Option<String> {
        let missing = self.missing_fields();
        if missing.is_empty() { return None; }
        match self {
            Self::Transfer { .. } => {
                if missing.contains(&"员工姓名".to_string()) && missing.contains(&"目标部门".to_string()) {
                    Some("请告诉我要调动哪位员工，调到哪个部门？".to_string())
                } else if missing.contains(&"员工姓名".to_string()) {
                    Some("请告诉我要调动哪位员工？".to_string())
                } else {
                    Some("请告诉我要调到哪个部门？".to_string())
                }
            }
            Self::CreateDepartment { .. } => Some("请指定新部门的名称。".to_string()),
            _ => None,
        }
    }
}

impl fmt::Display for ParsedIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transfer { employee_name, from_department, to_department } => {
                let emp = employee_name.as_deref().unwrap_or("?");
                let from = from_department.as_deref().unwrap_or("当前部门");
                let to = to_department.as_deref().unwrap_or("?");
                write!(f, "将 {emp} 从 {from} 调到 {to}")
            }
            Self::CreateDepartment { department_name, parent_department } => {
                let dept = department_name.as_deref().unwrap_or("?");
                match parent_department {
                    Some(parent) => write!(f, "在 {parent} 下新建部门 {dept}"),
                    None => write!(f, "新建部门 {dept}"),
                }
            }
            Self::Search { keyword, scope } => {
                let scope_text = match scope {
                    SearchScope::Employee => "员工",
                    SearchScope::Department => "部门",
                    SearchScope::All => "",
                };
                write!(f, "搜索{scope_text}：{keyword}")
            }
            Self::Cancel => write!(f, "取消当前操作"),
            Self::Unknown { message } => write!(f, "未识别：{message}"),
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transfer_full() {
        let result = parse_intent("把张三从市场部调到产品部").unwrap();
        match result.intent {
            ParsedIntent::Transfer { employee_name, from_department, to_department } => {
                assert_eq!(employee_name.as_deref(), Some("张三"));
                assert_eq!(from_department.as_deref(), Some("市场部"));
                assert_eq!(to_department.as_deref(), Some("产品部"));
            }
            _ => panic!("Expected Transfer intent"),
        }
    }

    #[test]
    fn test_parse_transfer_partial() {
        let result = parse_intent("将张三调到产品部").unwrap();
        match result.intent {
            ParsedIntent::Transfer { employee_name, to_department, .. } => {
                assert_eq!(employee_name.as_deref(), Some("张三"));
                assert_eq!(to_department.as_deref(), Some("产品部"));
            }
            _ => panic!("Expected Transfer intent"),
        }
    }

    #[test]
    fn test_parse_create_department() {
        let result = parse_intent("新建部门 AI实验室 放在技术中心下面").unwrap();
        match result.intent {
            ParsedIntent::CreateDepartment { department_name, parent_department } => {
                assert_eq!(department_name.as_deref(), Some("AI实验室"));
                assert_eq!(parent_department.as_deref(), Some("技术中心"));
            }
            _ => panic!("Expected CreateDepartment intent"),
        }
    }

    #[test]
    fn test_parse_search() {
        let result = parse_intent("搜索张三").unwrap();
        match result.intent {
            ParsedIntent::Search { keyword, .. } => assert_eq!(keyword, "张三"),
            _ => panic!("Expected Search intent"),
        }
    }

    #[test]
    fn test_parse_cancel() {
        let result = parse_intent("取消").unwrap();
        assert!(matches!(result.intent, ParsedIntent::Cancel));
    }

    #[test]
    fn test_missing_fields() {
        let result = parse_intent("调到产品部").unwrap();
        assert!(!result.missing_fields.is_empty());
    }

    #[test]
    fn test_llm_json_deserialization() {
        // 测试 LLM 返回的 JSON 能被正确解析
        let json = r#"{"type":"transfer","employee_name":"张三","from_department":"市场部","to_department":"产品部"}"#;
        let result = parse_llm_json(json).unwrap();
        match result.intent {
            ParsedIntent::Transfer { employee_name, from_department, to_department } => {
                assert_eq!(employee_name.as_deref(), Some("张三"));
                assert_eq!(from_department.as_deref(), Some("市场部"));
                assert_eq!(to_department.as_deref(), Some("产品部"));
            }
            _ => panic!("Expected Transfer"),
        }
        assert!(result.missing_fields.is_empty());
    }

    #[test]
    fn test_llm_json_with_nulls() {
        let json = r#"{"type":"transfer","employee_name":null,"from_department":null,"to_department":"产品部"}"#;
        let result = parse_llm_json(json).unwrap();
        assert_eq!(result.missing_fields.len(), 1);
        assert_eq!(result.missing_fields[0], "员工姓名");
    }

    #[test]
    fn test_llm_json_create_dept() {
        let json = r#"{"type":"create_department","department_name":"AI实验室","parent_department":"技术中心"}"#;
        let result = parse_llm_json(json).unwrap();
        match result.intent {
            ParsedIntent::CreateDepartment { department_name, parent_department } => {
                assert_eq!(department_name.as_deref(), Some("AI实验室"));
                assert_eq!(parent_department.as_deref(), Some("技术中心"));
            }
            _ => panic!("Expected CreateDepartment"),
        }
    }

    #[test]
    fn test_llm_json_with_markdown_wrapper() {
        let json = r#"```json
{"type":"cancel"}
```"#;
        let result = parse_llm_json(json).unwrap();
        assert!(matches!(result.intent, ParsedIntent::Cancel));
    }

    #[test]
    fn test_llm_json_search() {
        let json = r#"{"type":"search","keyword":"张三","scope":"employee"}"#;
        let result = parse_llm_json(json).unwrap();
        match result.intent {
            ParsedIntent::Search { keyword, scope } => {
                assert_eq!(keyword, "张三");
                assert_eq!(scope, SearchScope::Employee);
            }
            _ => panic!("Expected Search"),
        }
    }

    #[test]
    fn test_llm_json_unknown() {
        let json = r#"{"type":"unknown","message":"今天天气怎么样"}"#;
        let result = parse_llm_json(json).unwrap();
        match result.intent {
            ParsedIntent::Unknown { message } => assert_eq!(message, "今天天气怎么样"),
            _ => panic!("Expected Unknown"),
        }
    }
}