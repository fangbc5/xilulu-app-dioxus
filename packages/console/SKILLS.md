# Xilulu Console Skills

> 本文件定义了 xilulu-console 所有可用的操作技能（Skill）。
> 任何 LLM 均可读取此文件来理解系统支持的能力、参数和执行流程。

## 格式说明

每个 Skill 包含：
- **id**: 唯一标识符
- **name**: 人类可读名称
- **intent**: 意图类型标识
- **triggers**: 触发关键词列表
- **params**: 参数定义（名称 → 来源类型）
- **steps**: 执行步骤
- **api**: 实际调用的 API 端点

参数来源类型：
| type | 说明 |
|------|------|
| `user_input` | 从用户原始输入中提取 |
| `fixed` | 固定值，直接使用 `value` 字段 |
| `ask` | 需要追问用户，使用 `prompt` 字段提示 |
| `context` | 从上下文中获取，使用 `key` 字段查找 |

---

## Skill 列表

### builtin_create_dept_simple — 创建部门

- **intent**: `create_department`
- **triggers**: `新建部门`, `创建部门`, `添加部门`

**参数**:
| 参数名 | 来源 |
|--------|------|
| `department_name` | `user_input` — 部门名称 |

**执行步骤**:
1. 从用户输入中解析部门名称
2. 生成部门编码（拼音首字母缩写）
3. 调用 API: `POST /api/v1/team/departments` 创建部门

**API 请求体**:
```json
{
  "name": "<department_name>",
  "code": "<auto_generated>",
  "parent_id": null
}
```

**成功响应**: `✅ 部门「{name}」创建成功`

---

### builtin_create_dept_full — 创建部门（指定上级）

- **intent**: `create_department`
- **triggers**: `新建部门`, `创建部门`

**参数**:
| 参数名 | 来源 |
|--------|------|
| `department_name` | `user_input` — 部门名称 |
| `parent_department` | `user_input` — 上级部门名称 |

**执行步骤**:
1. 从用户输入中解析部门名称和上级部门名称
2. 调用 `GET /api/v1/team/departments` 查找上级部门 ID
3. 生成部门编码
4. 调用 API: `POST /api/v1/team/departments` 创建部门

**API 请求体**:
```json
{
  "name": "<department_name>",
  "code": "<auto_generated>",
  "parent_id": "<parent_dept_id>"
}
```

**成功响应**: `✅ 部门「{name}」创建成功，上级部门: {parent}`

---

### builtin_transfer — 员工调岗

- **intent**: `transfer`
- **triggers**: `调到`, `调岗`, `转到`, `调动`

**参数**:
| 参数名 | 来源 |
|--------|------|
| `employee_name` | `user_input` — 员工姓名 |
| `to_department` | `user_input` — 目标部门名称 |
| `from_department` | `user_input`（可选）— 原部门名称 |

**执行步骤**:
1. 从用户输入中解析员工姓名和目标部门
2. 调用 `GET /api/v1/team/departments` 查找目标部门 ID
3. 调用 `GET /api/v1/team/employees?keyword={name}` 查找员工 ID
4. 调用 API: `PUT /api/v1/team/employees/{id}/transfer` 执行调岗

**API 请求体**:
```json
{
  "department_id": "<target_dept_id>"
}
```

**成功响应**: `✅ 员工「{name}」已调至 {target_dept}`

---

### builtin_search — 搜索

- **intent**: `search`
- **triggers**: `搜索`, `查找`, `找`, `查询`

**参数**:
| 参数名 | 来源 |
|--------|------|
| `keyword` | `user_input` — 搜索关键词 |

**执行步骤**:
1. 从用户输入中提取搜索关键词
2. 同时调用:
   - `GET /api/v1/team/employees?keyword={keyword}` 搜索员工
   - `GET /api/v1/team/departments?keyword={keyword}` 搜索部门
3. 合并结果展示

**成功响应**: 搜索结果列表

---

### builtin_cancel — 取消操作

- **intent**: `cancel`
- **triggers**: `取消`, `算了`, `不要了`

**参数**: 无

**执行步骤**:
1. 重置对话状态为 Idle

**成功响应**: `已取消当前操作`

---

## 意图解析规则

当用户输入无法直接匹配时，使用以下规则推断意图：

| 用户输入模式 | 意图 | 示例 |
|-------------|------|------|
| `新建/创建/添加` + 名称 + `部门` | `create_department` | "新建AI实验室部门" |
| `把/将` + 人名 + `调到/转到` + 部门 | `transfer` | "把张三调到产品部" |
| `搜索/查找/找` + 关键词 | `search` | "搜索张三" |
| `取消/算了/不要了` | `cancel` | "取消" |

## API 基础信息

- **Base URL**: `http://localhost:30101`
- **Content-Type**: `application/json`
- **认证**: 暂未实现（计划使用 Bearer Token）