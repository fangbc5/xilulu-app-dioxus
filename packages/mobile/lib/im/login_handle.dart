import 'package:flutter/material.dart';
import 'dart:convert';
import 'package:get/get.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/pages/login/login_begin_page.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_api;
import 'package:wechat_flutter/tools/event/im_event.dart';

class ImLoginManager {
  static const int expireTime = 604800;

  static Future<void> init(BuildContext context) async {
    print("ImLoginManager init");
  }

  static Future<void> login(String userName, BuildContext context) async {
    print("ImLoginManager login - 开始初始化 Rust WebSocket 连接");
    await SharedUtil.instance.saveBoolean(Keys.hasLogged, true);
    
    try {
      final token = await SharedUtil.instance.getString('access_token') ?? '';
      final refreshToken = await SharedUtil.instance.getString('refresh_token') ?? '';
      if (token.isNotEmpty) {
        // ws://127.0.0.1:8080/ws 或使用部署好的网关地址，暂时默认网关或本地直接端口
        const String wsUrl = "ws://127.0.0.1:8080/ws";
        final String clientId = "flutter_client_${DateTime.now().millisecondsSinceEpoch}";
        
        // Smuggle both tokens to Rust without breaking FFI signature
        final String smuggledTokenPayload = jsonEncode({
          'access': token,
          'refresh': refreshToken,
        });

        await rust_api.coreStartWs(url: wsUrl, token: smuggledTokenPayload, clientId: clientId);
        
        // 绑定消息事件流 (现在从 Rust 传过来的是 JSON 字符串)
        final stream = await rust_api.coreSubscribeImEvents();
        stream.listen((eventStr) {
          debugPrint("收到远端 WS 消息事件(JSON): $eventStr");
          try {
            final Map<String, dynamic> eventJson = jsonDecode(eventStr) as Map<String, dynamic>;
            // 假设序列化后是 {"OnNewMessageReceived": { ...xmsg... }}
            if (eventJson.containsKey('OnNewMessageReceived')) {
              final msgMap = eventJson['OnNewMessageReceived'];
              final String roomId = msgMap['room_id']?.toString() ?? '';
              
              if (roomId.isNotEmpty) {
                // 通知 Flutter 更新对应聊天视框
                eventBusNewMsg.value = EventBusNewMsg(roomId);
                Notice.send(WeChatActions.msg(), '');
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
    await SharedUtil.instance.saveBoolean(Keys.hasLogged, false);
    await SharedUtil.instance.saveString('access_token', "");
    
    // 返回到登录界面大厅
    Get.offAll(() => LoginBeginPage());
  }

  static void addConversationListener() {}
}
