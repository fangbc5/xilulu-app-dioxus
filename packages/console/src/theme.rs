//! 主题系统
//!
//! 通过 CSS 自定义属性 + `data-theme` 属性实现主题切换。
//! 组件不直接引用颜色值，只引用 `var(--ds-*)` 变量。
//!
//! ## 使用方式
//!
//! ```rust
//! // 在根组件中
//! use_context_provider(|| ThemeState::new(Theme::Jade));
//!
//! // 切换主题
//! let mut theme_state = use_context::<ThemeState>();
//! theme_state.set(Theme::DeepSpace);
//! ```

use dioxus::prelude::*;

/// 可用主题
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    /// 翡翠绿 — 清爽白底 + 绿色强调
    Jade,
    /// 深空蓝 — 深色背景 + 极光渐变
    DeepSpace,
    /// 黑曜石 — 纯黑简约
    Obsidian,
}

impl Theme {
    /// 返回 CSS data-theme 属性值
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Jade => "jade",
            Theme::DeepSpace => "deep-space",
            Theme::Obsidian => "obsidian",
        }
    }

    /// 显示名称
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Jade => "翡翠绿",
            Theme::DeepSpace => "深空蓝",
            Theme::Obsidian => "黑曜石",
        }
    }

    /// 所有可用主题
    pub fn all() -> &'static [Theme] {
        &[Theme::Jade, Theme::DeepSpace, Theme::Obsidian]
    }

    /// 下一个主题（循环切换）
    pub fn next(&self) -> Theme {
        match self {
            Theme::Jade => Theme::DeepSpace,
            Theme::DeepSpace => Theme::Obsidian,
            Theme::Obsidian => Theme::Jade,
        }
    }
}

/// 主题全局状态
#[derive(Clone, Copy)]
pub struct ThemeState {
    current: Signal<Theme>,
}

impl ThemeState {
    /// 创建主题状态（默认主题）
    pub fn new(initial: Theme) -> Self {
        Self {
            current: Signal::new(initial),
        }
    }

    /// 获取当前主题
    pub fn current(&self) -> Theme {
        (self.current)()
    }

    /// 设置主题
    pub fn set(&mut self, theme: Theme) {
        self.current.set(theme);
    }

    /// 切换到下一个主题
    pub fn toggle(&mut self) {
        let next = self.current().next();
        self.current.set(next);
    }

    /// 返回 data-theme 值
    pub fn data_attr(&self) -> &'static str {
        self.current().as_str()
    }
}

/// 主题 Provider 组件
///
/// 在应用根部挂载，将 `data-theme` 属性应用到根容器上。
/// 所有子组件通过 CSS 变量自动获取当前主题色彩。
#[component]
pub fn ThemeProvider(
    /// 初始主题
    #[props(default = Theme::Jade)]
    initial: Theme,
    /// 子元素
    children: Element,
) -> Element {
    let theme_state = use_hook(|| ThemeState::new(initial));
    use_context_provider(|| theme_state);

    let theme_attr = theme_state.data_attr();

    rsx! {
        div {
            "data-theme": "{theme_attr}",
            style: "width: 100%; height: 100%; display: flex; flex-direction: column;",
            {children}
        }
    }
}

/// 主题切换按钮组件
///
/// 点击循环切换主题，显示当前主题标签。
#[component]
pub fn ThemeToggle() -> Element {
    let mut theme_state = use_context::<ThemeState>();
    let label = theme_state.current().label();

    rsx! {
        button {
            onclick: move |_| theme_state.toggle(),
            style: "
                display: inline-flex; align-items: center; gap: 6px;
                padding: 4px 10px; border-radius: 6px;
                background: var(--ds-bg-surface); border: 1px solid var(--ds-border);
                color: var(--ds-text-secondary); font-size: 12px; font-family: inherit;
                cursor: pointer; transition: all 120ms ease;
            ",
            span { style: "font-size: 10px;", "●" }
            span { "{label}" }
        }
    }
}
