use crate::i18n::use_language;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBellOff, LdCirclePlus, LdMonitor, LdSearch, LdStar};
use dioxus_free_icons::Icon;
use rust_i18n::t;

const AVATAR_COLORS: &[&str] = &[
    "#1890ff", "#52c41a", "#fa8c16", "#eb2f96", "#722ed1", "#13c2c2", "#f5222d",
];

#[component]
pub fn AppChat() -> Element {
    let lang = use_language();
    let l_str = lang().as_str();

    let core_app = use_context::<std::sync::Arc<xilulu_im_sdk::service::app::CoreApp>>();
    let mut rooms = use_signal(|| core_app.chat.rooms_receiver.borrow().clone());

    use_effect({
        let chat_service = core_app.chat.clone();
        move || {
            let chat_clone = chat_service.clone();
            spawn(async move {
                if chat_clone.rooms_receiver.borrow().is_empty() {
                    let _ = chat_clone.load_rooms().await;
                }
            });
        }
    });

    use_effect({
        let rx = core_app.chat.rooms_receiver.clone();
        move || {
            let mut my_rx = rx.clone();
            spawn(async move {
                while my_rx.changed().await.is_ok() {
                    let new_rooms = my_rx.borrow().clone();
                    rooms.set(new_rooms);
                }
            });
        }
    });

    rsx! {
        div { class: "w-full h-full flex flex-col bg-[#ededed] dark:bg-black pt-[env(safe-area-inset-top)]",

            // 微信风格 Header
            // - 背景：#ededed（浅灰）
            // - 标题：居中加粗
            // - 右侧：带圈的加号
            header { class: "flex-shrink-0 h-[44px] px-4 flex items-center justify-between transition-colors bg-[#ededed] dark:bg-zinc-900 border-none",
                div { class: "w-10 flex items-center justify-start text-zinc-900 dark:text-zinc-100",
                    button { class: "active:opacity-50 transition-opacity outline-none",
                        Icon { width: 22, height: 22, icon: LdStar }
                    }
                }
                div { class: "flex-1 flex justify-center",
                    span { class: "text-[17px] font-semibold text-zinc-900 dark:text-zinc-100",
                        "{t!(\"nav.chat\", locale = l_str)}"
                    }
                }
                div { class: "w-10 flex items-center justify-end text-zinc-900 dark:text-zinc-100",
                    button { class: "active:opacity-50 transition-opacity outline-none",
                        Icon { width: 22, height: 22, icon: LdCirclePlus }
                    }
                }
            }

            // 列表区域
            main { class: "flex-1 overflow-x-hidden overflow-y-auto overscroll-contain bg-white dark:bg-black pb-[calc(50px+env(safe-area-inset-bottom))]",

                // 搜索栏区域 (随内容滑动)
                div { class: "px-2 pb-2 pt-1 bg-[#ededed] dark:bg-zinc-900 border-none",
                    div { class: "w-full h-[36px] bg-white dark:bg-zinc-800 rounded flex items-center justify-center gap-1.5 text-[#b2b2b2] dark:text-zinc-500",
                        Icon { width: 16, height: 16, icon: LdSearch }
                        span { class: "text-[15px]",
                            "{t!(\"contact.searchPlaceholder\", locale = l_str)}"
                        }
                    }
                }

                // Mac 微信已登录 提示条
                div { class: "w-full h-[40px] bg-[#f5f5f5] dark:bg-zinc-900 flex items-center px-4 border-b border-zinc-100 dark:border-zinc-800/50",
                    div { class: "w-12 mr-3 flex items-center justify-center flex-shrink-0 relative right-1",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdMonitor,
                            class: "text-[#555555] dark:text-zinc-500",
                        }
                    }
                    span { class: "text-[13px] text-[#555555] dark:text-zinc-500", "Mac 微信已登录" }
                }

                {
                    let nodes: Vec<_> = rooms()

                        .into_iter()
                        .map(|room| {
                            let dynamic_bg = if room.is_top {
                                "bg-[#f3f3f3] dark:bg-[#111111]"
                            } else {
                                "bg-white dark:bg-black"
                            };
                            let mut hash: usize = 0;
                            for byte in room.id.bytes() {
                                hash = hash.wrapping_add(byte as usize);
                            }
                            let bg_color = format!(
                                "background-color: {}",
                                AVATAR_COLORS[hash % AVATAR_COLORS.len()],
                            );
                            rsx! {
                                div {
                                    key: "{room.id}",
                                    class: "w-full flex items-center h-[72px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer {dynamic_bg}",
                
                                    div { class: "relative flex-shrink-0 w-12 h-12 mr-3 flex items-center justify-center text-white text-lg font-medium",
                                        if let Some(av) = &room.avatar {
                                            img {
                                                src: "{av}",
                                                class: "w-full h-full object-cover rounded-[10px]",
                                            }
                                        } else {
                                            div {
                                                class: "w-full h-full flex items-center justify-center rounded-[10px]",
                                                style: "{bg_color}",
                                                "{room.name.chars().next().unwrap_or('?')}"
                                            }
                                        }
                
                                        if room.unread > 0 {
                                            if room.is_mute {
                                                div { class: "absolute -top-1 -right-1 w-[10px] h-[10px] bg-[#f5222d] border border-white dark:border-zinc-900 rounded-full" }
                                            } else {
                                                div { class: "absolute -top-1.5 -right-1.5 min-w-[18px] h-[18px] rounded-full flex items-center justify-center px-[4px] bg-[#f5222d] border-[1.5px] border-white dark:border-zinc-900 text-[10px] font-bold text-white",
                                                    {if room.unread > 99 { "99+".to_string() } else { room.unread.to_string() }}
                                                }
                                            }
                                        }
                                    }
                
                                    div { class: "flex-1 h-full py-[10px] flex flex-col justify-between border-b border-[#f3f3f3] dark:border-zinc-800/50",
                
                                        div { class: "w-full flex items-center justify-between pr-4",
                                            span { class: "flex-1 min-w-0 truncate text-[16.5px] text-zinc-900 dark:text-zinc-100 font-normal pr-2",
                                                "{room.name}"
                                            }
                                            span { class: "flex-shrink-0 text-[12px] text-[#b2b2b2] dark:text-zinc-500",
                                                "{room.last_time}"
                                            }
                                        }
                
                                        div { class: "w-full flex items-center justify-between pr-4",
                                            p { class: "flex-1 min-w-0 truncate text-[14px] text-[#b2b2b2] dark:text-zinc-500 pr-2",
                                                if room.last_msg.starts_with("[群公告]") {
                                                    span { class: "text-[#fa5151]", "[群公告]" }
                                                    span { {room.last_msg.replace("[群公告]", "")} }
                                                } else {
                                                    "{room.last_msg}"
                                                }
                                            }
                                            if room.is_mute {
                                                Icon {
                                                    icon: LdBellOff,
                                                    width: 14,
                                                    height: 14,
                                                    class: "flex-shrink-0 text-[#d4d4d4] dark:text-zinc-600",
                                                }
                                            } else {
                                                div { class: "w-[14px]" } // 占位保持高度对齐
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .collect();
                    rsx! {
                        {nodes.into_iter()}
                    }
                }
            }
        }
    }
}
