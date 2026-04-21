import 'package:flutter/material.dart';
import 'dart:convert';
import 'dart:math';
import 'package:get/get.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/pages/login/login_begin_page.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_api;
import 'package:wechat_flutter/tools/event/im_event.dart';
import 'package:provider/provider.dart';
import 'package:wechat_flutter/provider/global_model.dart';

String _generateUuidV4() {
  final random = Random.secure();
  final values = List<int>.generate(16, (i) => random.nextInt(256));
  // RFC 4122 version 4
  values[6] = (values[6] & 0x0f) | 0x40;
  // RFC 4122 variant
  values[8] = (values[8] & 0x3f) | 0x80;
  final hex = values.map((b) => b.toRadixString(16).padLeft(2, '0')).toList();
  return '${hex.sublist(0, 4).join('')}-${hex.sublist(4, 6).join('')}-${hex.sublist(6, 8).join('')}-${hex.sublist(8, 10).join('')}-${hex.sublist(10, 16).join('')}';
}

class ImLoginManager {
  static const int expireTime = 604800;
  static bool _isConnecting = false;

  static Future<void> init(BuildContext context) async {
    print("ImLoginManager init");
  }

  static Future<void> login(String userName, BuildContext context) async {
    if (_isConnecting) {
      print("ImLoginManager login - 已经在连接中，跳过重复初始化");
      return;
    }
    _isConnecting = true;
    print("ImLoginManager login - 开始初始化 Rust WebSocket 连接");
    await SharedUtil.instance.saveBoolean(Keys.hasLogged, true);
    
    try {
      final token = await SharedUtil.instance.getString('access_token') ?? '';
      final refreshToken = await SharedUtil.instance.getString('refresh_token') ?? '';
      if (token.isNotEmpty) {
        // ws://127.0.0.1:8080/ws 或使用部署好的网关地址，暂时默认网关或本地直接端口
        const String wsUrl = "ws://127.0.0.1:8080/ws";
        
        // 【业界标准】获取或生成设备的唯一标识符 (UUID v4)
        String clientId = await SharedUtil.instance.getString('device_client_id') ?? '';
        if (clientId.isEmpty) {
          clientId = _generateUuidV4();
          await SharedUtil.instance.saveString('device_client_id', clientId);
        }
        print("========== FINAL CLIENT ID SENT TO RUST: $clientId ==========");
        
        final bool syncChatHistory = await SharedUtil.instance.getBoolean('sync_chat_history') ?? true;
        // Smuggle both tokens and flag to Rust without breaking FFI signature
        final String smuggledTokenPayload = jsonEncode({
          'access': token,
          'refresh': refreshToken,
          'sync_chat_history': syncChatHistory,
        });

        await rust_api.coreStartWs(url: wsUrl, token: smuggledTokenPayload, clientId: clientId);
        
        // 绑定消息事件流 (现在从 Rust 传过来的是 JSON 字符串)
        final stream = await rust_api.coreSubscribeImEvents();
        stream.listen((eventStr) {
          debugPrint("收到远端 WS 消息事件(JSON): $eventStr");
          try {
            final dynamic decoded = jsonDecode(eventStr);
            
            if (decoded is String) {
              if (decoded == 'OnConversationListUpdated') {
                // Trigger reload of contacts and conversations
                eventBusNewMsg.value = EventBusNewMsg('GLOBAL_SYNC_COMPLETE');
                Notice.send(WeChatActions.msg(), '');
              }
            } else if (decoded is Map<String, dynamic>) {
              if (decoded.containsKey('DEBUG_SYNC')) {
                print("====================================");
                print("🟢 RUST SYNC DEBUG: ${decoded['DEBUG_SYNC']}");
                print("====================================");
              } else if (decoded.containsKey('OnNewMessageReceived')) {
                final msgMap = decoded['OnNewMessageReceived'];
                final String roomId = msgMap['room_id']?.toString() ?? '';
                
                if (roomId.isNotEmpty) {
                  // 通知 Flutter 更新对应聊天视框
                  eventBusNewMsg.value = EventBusNewMsg(roomId);
                  Notice.send(WeChatActions.msg(), '');
                }
              } else if (decoded.containsKey('OnChatLatestMessageUpdated')) {
                final msgMap = decoded['OnChatLatestMessageUpdated'];
                final String roomId = msgMap['room_id']?.toString() ?? '';
                final String content = msgMap['content']?.toString() ?? '';
                final int time = msgMap['created_at'] ?? 0;
                
                if (roomId.isNotEmpty) {
                  // 局部静默更新推送
                  eventBusNewMsg.value = EventBusNewMsg('LATEST_UPDATE_$roomId');
                  Notice.send(WeChatActions.msg(), {
                    'type': 'LATEST_UPDATE',
                    'room_id': roomId,
                    'content': content,
                    'time': time
                  });
                }
              }
            }
          } catch (e) {
            debugPrint("解析 WS 事件失败: $e");
          }
        });
        print("WebSocket 已成功连接并挂载事件流");
      } else {
        print("警告: access_token 为空，无法建立 WS 连接");
      }
    } catch (e) {
      print("WebSocket 启动失败: $e");
    }
  }

  static Future<void> loginOut(BuildContext context) async {
    print("ImLoginManager loginOut - clearing state");
    _isConnecting = false;
    await SharedUtil.instance.saveBoolean(Keys.hasLogged, false);
    await SharedUtil.instance.saveString('access_token', "");
    
    // 返回到登录界面大厅
    Get.offAll(() => LoginBeginPage());
  }

  static void addConversationListener() {}
}
