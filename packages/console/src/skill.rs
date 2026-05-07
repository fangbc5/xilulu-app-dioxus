//! Skill 系统
//!
//! 当一个对话流程成功执行后，自动录制为标准 Skill。
//! 下次相同意图触发时，直接按标准流程执行，无需再走 LLM 解析。
//!
//! 持久化策略：
//! - 预置 Skill 定义在 `SKILLS.md` 中，LLM 可直接读取
//! - 运行时使用记录保存在浏览器 localStorage（key: `xilulu_skill_store`）
//! - 支持导出为 Markdown 格式供任意 LLM 使用

use crate::intent::ParsedIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 一个标准化的技能
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    /// 唯一 ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 意图类型
    pub intent_type: String,
    /// 触发关键词
    pub trigger_keywords: Vec<String>,
    /// 参数模板
    pub param_templates: HashMap<String, ParamSource>,
    /// 执行步骤描述
    pub steps: Vec<String>,
    /// 使用次数
    pub use_count: u32,
    /// 最后使用时间
    pub last_used: Option<String>,
}

/// 参数来源
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParamSource {
    UserInput,
    Fixed { value: String },
    Ask { prompt: String },
    Context { key: String },
}

/// localStorage key
const STORAGE_KEY: &str = "xilulu_skill_store";

/// Skill 管理器
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillStore {
    pub skills: Vec<Skill>,
}

impl SkillStore {
    /// 创建并加载（预置 + localStorage 持久化记录）
    pub fn new() -> Self {
        let mut store = Self::default();
        store.add_builtin_skills();
        store.load_from_storage();
        store
    }

    /// 从 localStorage 加载使用记录
    fn load_from_storage(&mut self) {
        // WASM 环境下从 localStorage 读取
        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = web_sys::window() else { return };
            let Ok(storage) = window.local_storage() else { return };
            let Some(storage) = storage else { return };
            let Ok(Some(json)) = storage.get_item(STORAGE_KEY) else { return };

            if let Ok(record) = serde_json::from_str::<SkillUsageRecord>(&json) {
                // 合并持久化的使用次数到预置 Skill
                for skill in &mut self.skills {
                    if let Some(count) = record.use_counts.get(&skill.id) {
                        skill.use_count = *count;
                    }
                    if let Some(time) = record.last_used.get(&skill.id) {
                        skill.last_used = Some(time.clone());
                    }
                }
            }
        }
    }

    /// 保存使用记录到 localStorage
    fn save_to_storage(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut record = SkillUsageRecord::default();
            for skill in &self.skills {
                record.use_counts.insert(skill.id.clone(), skill.use_count);
                if let Some(t) = &skill.last_used {
                    record.last_used.insert(skill.id.clone(), t.clone());
                }
            }
            if let Ok(json) = serde_json::to_string(&record) {
                let Some(window) = web_sys::window() else { return };
                let Ok(storage) = window.local_storage() else { return };
                let Some(storage) = storage else { return };
                let _ = storage.set_item(STORAGE_KEY, &json);
            }
        }
    }

    /// 导出所有 Skill 为 Markdown 格式（供 LLM 读取）
    pub fn export_as_markdown(&self) -> String {
        let mut md = String::from("# Xilulu Console Skills\n\n");
        md.push_str("> 自动生成的 Skill 列表，LLM 可直接读取。\n\n");

        for skill in &self.skills {
            md.push_str(&format!("## {}\n\n", skill.id));
            md.push_str(&format!("- **名称**: {}\n", skill.name));
            md.push_str(&format!("- **意图**: `{}`\n", skill.intent_type));
            md.push_str(&format!("- **触发词**: {}\n", skill.trigger_keywords.join(", ")));
            md.push_str(&format!("- **使用次数**: {}\n\n", skill.use_count));

            md.push_str("**参数**:\n");
            for (name, source) in &skill.param_templates {
                md.push_str(&format!("- `{}`: {:?}\n", name, source));
            }

            md.push_str("\n**执行步骤**:\n");
            for step in &skill.steps {
                md.push_str(&format!("{step}\n"));
            }
            md.push('\n');
        }

        md
    }

    fn add_builtin_skills(&mut self) {
        self.skills.push(Skill {
            id: "builtin_create_dept_full".into(),
            name: "创建部门（指定上级）".into(),
            intent_type: "create_department".into(),
            trigger_keywords: vec!["新建部门".into(), "创建部门".into()],
            param_templates: {
                let mut m = HashMap::new();
                m.insert("department_name".into(), ParamSource::UserInput);
                m.insert("parent_department".into(), ParamSource::UserInput);
                m
            },
            steps: vec![
                "1️⃣ 解析部门名称和上级部门".into(),
                "2️⃣ 查找上级部门 ID".into(),
                "3️⃣ 调用 API 创建部门".into(),
            ],
            use_count: 0,
            last_used: None,
        });

        self.skills.push(Skill {
            id: "builtin_create_dept_simple".into(),
            name: "创建部门".into(),
            intent_type: "create_department".into(),
            trigger_keywords: vec!["新建部门".into(), "创建部门".into(), "添加部门".into()],
            param_templates: {
                let mut m = HashMap::new();
                m.insert("department_name".into(), ParamSource::UserInput);
                m
            },
            steps: vec![
                "1️⃣ 解析部门名称".into(),
                "2️⃣ 调用 API 创建部门".into(),
            ],
            use_count: 0,
            last_used: None,
        });

        self.skills.push(Skill {
            id: "builtin_transfer".into(),
            name: "员工调岗".into(),
            intent_type: "transfer".into(),
            trigger_keywords: vec!["调到".into(), "调岗".into(), "转到".into()],
            param_templates: {
                let mut m = HashMap::new();
                m.insert("employee_name".into(), ParamSource::UserInput);
                m.insert("to_department".into(), ParamSource::UserInput);
                m
            },
            steps: vec![
                "1️⃣ 解析员工和部门信息".into(),
                "2️⃣ 查找员工 ID".into(),
                "3️⃣ 调用 API 执行调岗".into(),
            ],
            use_count: 0,
            last_used: None,
        });
    }

    /// 根据意图匹配 Skill
    pub fn match_skill_by_intent(&self, intent: &ParsedIntent) -> Option<&Skill> {
        let target_id = match intent {
            ParsedIntent::CreateDepartment { parent_department: Some(_), .. } => "builtin_create_dept_full",
            ParsedIntent::CreateDepartment { .. } => "builtin_create_dept_simple",
            ParsedIntent::Transfer { .. } => "builtin_transfer",
            _ => return None,
        };
        self.skills.iter().find(|s| s.id == target_id)
    }

    /// 记录一次成功执行（同时持久化到 localStorage）
    pub fn record_success(&mut self, intent: &ParsedIntent) {
        let target_id = match intent {
            ParsedIntent::CreateDepartment { parent_department: Some(_), .. } => "builtin_create_dept_full",
            ParsedIntent::CreateDepartment { .. } => "builtin_create_dept_simple",
            ParsedIntent::Transfer { .. } => "builtin_transfer",
            ParsedIntent::Search { .. } => "builtin_search",
            _ => return,
        };
        for skill in &mut self.skills {
            if skill.id == target_id {
                skill.use_count += 1;
                skill.last_used = Some(now_timestamp());
                break;
            }
        }
        self.save_to_storage();
    }

    /// 按使用次数排序
    pub fn skills_by_usage(&self) -> Vec<&Skill> {
        let mut s: Vec<&Skill> = self.skills.iter().collect();
        s.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        s
    }

    /// 获取 SKILLS.md 内容（用于 LLM system prompt）
    /// 如果内置编译的 SKILLS.md 不存在，则用 export_as_markdown() 兜底
    pub fn skills_md_for_llm(&self) -> String {
        // 使用编译时 include_str 加载 SKILLS.md
        let builtin = include_str!("../SKILLS.md");
        if builtin.is_empty() {
            self.export_as_markdown()
        } else {
            // 追加运行时使用统计
            let mut result = builtin.to_string();
            result.push_str("\n\n---\n\n## 运行时使用统计\n\n");
            for skill in self.skills_by_usage() {
                result.push_str(&format!(
                    "- **{}** ({}): 使用 {} 次\n",
                    skill.id, skill.name, skill.use_count
                ));
            }
            result
        }
    }
}

/// 持久化到 localStorage 的使用记录
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
struct SkillUsageRecord {
    use_counts: HashMap<String, u32>,
    last_used: HashMap<String, String>,
}

fn now_timestamp() -> String {
    // WASM 环境下获取当前时间
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Date;
        let now = Date::new_0();
        format!(
            "{}/{:02}/{} {:02}:{:02}",
            now.get_full_year(),
            now.get_month() + 1,
            now.get_date(),
            now.get_hours(),
            now.get_minutes(),
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "刚刚".to_string()
    }
}
