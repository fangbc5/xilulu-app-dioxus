use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCompass, LdLayoutGrid, LdMessageCircle, LdUser, LdUsers,
};
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq, Props)]
pub struct BottomNavItem {
    pub label: String,
    pub icon: String,
    pub target_route: Route,
}

#[derive(Clone, PartialEq, Props)]
pub struct BottomNavProps {
    /// Option::None 表示加载中（显示骨架屏）；Some(vec) 表示数据就绪
    pub items: Option<Vec<BottomNavItem>>,
    #[props(default = "bg-[#f9f9f9]/90 dark:bg-[#1C1C1E]/90 backdrop-blur-md border-t border-black/5 dark:border-white/5".to_string())]
    pub bg_class: String,
    #[props(default = "text-[#07C160]".to_string())]
    pub active_color_class: String,
    #[props(default = "text-[#999999] dark:text-[#8E8E93]".to_string())]
    pub inactive_color_class: String,
    #[props(default = "暂无可用菜单".to_string())]
    pub empty_text: String,
}

/// 根据图标名称渲染对应的 Ionicons/Lucide Outline 图标
/// 微信风格：始终使用线性轮廓图标，仅通过父级颜色类区分激活态
fn render_icon(icon_name: &str) -> Element {
    let size = 24;

    match icon_name {
        "message" | "chat" | "wechat" => {
            rsx! {
                Icon { width: size, height: size, icon: LdMessageCircle }
            }
        }
        "contact" | "contacts" | "users" | "people" | "document" => {
            rsx! {
                Icon { width: size, height: size, icon: LdUsers }
            }
        }
        "social" | "discover" | "compass" | "home" => {
            rsx! {
                Icon { width: size, height: size, icon: LdCompass }
            }
        }
        "user" | "me" | "profile" => {
            rsx! {
                Icon { width: size, height: size, icon: LdUser }
            }
        }
        _ => {
            rsx! {
                Icon { width: size, height: size, icon: LdLayoutGrid }
            }
        }
    }
}

#[component]
pub fn BottomNav(props: BottomNavProps) -> Element {
    let current_route = use_route::<Route>();

    rsx! {
        div { class: "fixed bottom-0 left-0 right-0 z-50 pb-[env(safe-area-inset-bottom)] select-none {props.bg_class}",
            div { class: "flex justify-around items-stretch h-[50px]",

                match props.items.as_ref() {
                    Some(items) if !items.is_empty() => {
                        rsx! {
                            for item in items {
                                {
                                    let is_active = current_route == item.target_route;
                                    let color_class = if is_active {

            

                                        &props.active_color_class
                                    } else {
                                        &props.inactive_color_class
                                    };
                                    rsx! {
                                        Link {
                                            to: item.target_route.clone(),
                                            class: "flex flex-col items-center justify-center flex-1 outline-none [-webkit-tap-highlight-color:transparent] {color_class}",
                    
                                            {render_icon(&item.icon)}
                    
                                            span { class: "text-[10px] leading-none mt-[2px]", "{item.label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(_) => {
                        rsx! {
                            div { class: "w-full h-full flex items-center justify-center text-[#999999] text-xs",
                                "{props.empty_text}"
                            }
                        }
                    }
                    None => {
                        rsx! {
                            div { class: "w-full h-full flex items-center justify-center text-[#999999] text-xs animate-pulse",
                                "Loading..."
                            }
                        }
                    }
                }
            }
        }
    }
}
