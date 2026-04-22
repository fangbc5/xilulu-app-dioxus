import 'dart:convert';
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/pages/login/login_begin_page.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_api;
import 'package:wechat_flutter/tools/event/im_event.dart';
import 'package:wechat_flutter/config/storage_manager.dart';

/// 生成 RFC 4122 v4 UUID，用于设备唯一标识
String _generateUuidV4() {
  final random = Random.secure();
  final values = List<int>.generate(16, (i) => random.nextInt(256));
  values[6] = (values[6] & 0x0f) | 0x40;
  values[8] = (values[8] & 0x3f) | 0x80;
  final hex = values.map((b) => b.toRadixString(16).padLeft(2, '0')).toList();
  return '${hex.sublist(0, 4).join('')}-${hex.sublist(4, 6).join('')}'
      '-${hex.sublist(6, 8).join('')}-${hex.sublist(8, 10).join('')}'
      '-${hex.sublist(10, 16).join('')}';
}

/// IM 会话管理器。
///
/// 职责单一：管理 WebSocket 会话的生命周期。
/// Token 由 Rust 侧 GLOBAL_STORAGE 持有，本类不直接操作 access_token / refresh_token。
class ImLoginManager {
  static const String _wsUrl = "ws://127.0.0.1:8080/ws";
  static bool _wsStarted = false;

  /// 冷启动检查并向底层注入核心 Token
  /// 严格查验 Token 有效期，一旦失效立马清空，不传入底层 SDK，保证不引发网络雪崩。
  static Future<void> checkAndInjectTokens() async {
    await StorageManager.init(); // 确保 SharedUtil 已经实例化可用
    final savedAccess = await SharedUtil.instance.getString('access_token') ?? '';
    final savedRefresh = await SharedUtil.instance.getString('refresh_token') ?? '';

    if (savedAccess.isNotEmpty && savedRefresh.isNotEmpty) {
      final accessExpiresAt = await SharedUtil.instance.getInt('access_expires_at') ?? 0;
      final refreshExpiresAt = await SharedUtil.instance.getInt('refresh_expires_at') ?? 0;
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;

      if (refreshExpiresAt > 0 && now >= refreshExpiresAt) {
        debugPrint('🚫 冷启动拦截：refresh_token 本地时间查验已过期，强制清空并跳转登录');
        await SharedUtil.instance.saveBoolean(Keys.hasLogged, false);
        await SharedUtil.instance.saveString('access_token', '');
        await SharedUtil.instance.saveString('refresh_token', '');
      } else {
        final savedUserId = await SharedUtil.instance.getString(Keys.account) ?? '';
        await rust_api.coreUpdateTokens(
          accessToken: savedAccess,
          refreshToken: savedRefresh,
          accessExpiresAt: accessExpiresAt > 0 ? accessExpiresAt : null,
          refreshExpiresAt: refreshExpiresAt > 0 ? refreshExpiresAt : null,
          userId: savedUserId.isNotEmpty ? savedUserId : null,
        );
        debugPrint('🔄 Token 种子注入完成: access_expires_at=$accessExpiresAt, refresh_expires_at=$refreshExpiresAt, user_id=$savedUserId');
      }
    } else {
      debugPrint('🚫 冷启动拦截：本地无可用 Token 凭据，强制回退至登录态');
      await SharedUtil.instance.saveBoolean(Keys.hasLogged, false);
    }
  }

  /// 全局初始化事件流，只执行一次，供 Rust Daemon 投递 AUTH_EXPIRED 信号。
  static Future<void> initGlobalListener() async {
    final stream = await rust_api.coreSubscribeImEvents();
    stream.listen((eventStr) async {
      try {
        final dynamic decoded = jsonDecode(eventStr);
        if (decoded is Map<String, dynamic>) {
          await _handleMapEvent(decoded);
        } else if (decoded is String) {
          _handleStringEvent(decoded);
        }
      } catch (e) {
        debugPrint('解析全局事件失败: $e');
      }
    });
    debugPrint('✅ ImLoginManager: 全局事件流已挂载');
  }

  /// 启动 WS 连接并挂载事件监听器。
  /// 幂等：已启动时直接返回。
  static Future<void> startWs({bool syncChatHistory = true}) async {
    if (_wsStarted) return;
    _wsStarted = true;

    try {
      // 获取或生成设备唯一标识符
      String clientId = await SharedUtil.instance.getString('device_client_id') ?? '';
      if (clientId.isEmpty) {
        clientId = _generateUuidV4();
        await SharedUtil.instance.saveString('device_client_id', clientId);
      }

      await rust_api.coreStartWs(
        url: _wsUrl,
        clientId: clientId,
        syncChatHistory: syncChatHistory,
      );

      // 挂载事件流监听器
      final stream = await rust_api.coreSubscribeImEvents();
      stream.listen((eventStr) async {
        try {
          final dynamic decoded = jsonDecode(eventStr);
          if (decoded is Map<String, dynamic>) {
            await _handleMapEvent(decoded);
          } else if (decoded is String) {
            _handleStringEvent(decoded);
          }
        } catch (e) {
          debugPrint('解析 WS 事件失败: $e');
        }
      });

      debugPrint('✅ ImLoginManager: WebSocket 已成功连接并挂载事件流');
    } catch (e) {
      _wsStarted = false;
      debugPrint('❌ ImLoginManager: WebSocket 启动失败: $e');
    }
  }

  /// 处理 Map 类型的 WS 事件
  static Future<void> _handleMapEvent(Map<String, dynamic> decoded) async {
    if (decoded.containsKey('AUTH_EXPIRED')) {
      // Token 不可恢复，强制跳转登录页
      debugPrint('🔴 AUTH_EXPIRED: Token 已失效，正在跳转登录页...');
      await _forceLogout();
      return;
    }

    if (decoded.containsKey('TOKEN_REFRESHED')) {
      // Rust 侧自动刷新了 Token，同步持久化到 Flutter SharedPreferences
      final tokens = decoded['TOKEN_REFRESHED'] as Map<String, dynamic>?;
      if (tokens != null) {
        final access  = tokens['access']  as String? ?? '';
        final refresh = tokens['refresh'] as String? ?? '';
        final expiresIn = tokens['expires_in'] as int? ?? 900;

        if (access.isNotEmpty) {
          final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
          await SharedUtil.instance.saveString('access_token', access);
          await SharedUtil.instance.saveString('refresh_token', refresh);
          await SharedUtil.instance.saveInt('access_expires_at', now + expiresIn);
          // 注意：refresh_expires_at 不更新，因为 refresh_token 未变化
          debugPrint('🔄 TOKEN_REFRESHED: 新 Token 已持久化，expires_at=${now + expiresIn}');
        }
      }
      return;
    }

    if (decoded.containsKey('OnNewMessageReceived')) {
      final msgMap = decoded['OnNewMessageReceived'] as Map<String, dynamic>?;
      final roomId = msgMap?['room_id']?.toString() ?? '';
      if (roomId.isNotEmpty) {
        eventBusNewMsg.value = EventBusNewMsg(roomId);
        Notice.send(WeChatActions.msg(), '');
      }
      return;
    }

    if (decoded.containsKey('OnChatLatestMessageUpdated')) {
      final msgMap = decoded['OnChatLatestMessageUpdated'] as Map<String, dynamic>?;
      final roomId  = msgMap?['room_id']?.toString() ?? '';
      final content = msgMap?['content']?.toString() ?? '';
      final time    = msgMap?['created_at'] as int? ?? 0;
      if (roomId.isNotEmpty) {
        eventBusNewMsg.value = EventBusNewMsg('LATEST_UPDATE_$roomId');
        Notice.send(WeChatActions.msg(), {
          'type': 'LATEST_UPDATE',
          'room_id': roomId,
          'content': content,
          'time': time,
        });
      }
      return;
    }

    // WS 连接状态变更 → 通知 UI 更新连接指示条
    if (decoded.containsKey('OnConnectionStatusChanged')) {
      final status = decoded['OnConnectionStatusChanged'] as String? ?? '';
      if (status.isNotEmpty) {
        eventBusNewMsg.value = EventBusNewMsg('CONNECTION_STATUS_$status');
        Notice.send(WeChatActions.connectionStatus(), status);
      }
      return;
    }
  }

  /// 处理字符串类型的 WS 事件（枚举序列化）
  static void _handleStringEvent(String event) {
    if (event == 'OnConversationListUpdated') {
      eventBusNewMsg.value = EventBusNewMsg('GLOBAL_SYNC_COMPLETE');
      Notice.send(WeChatActions.msg(), '');
    } else if (event == 'OnSyncStarted') {
      // 增量同步开始 → UI 显示"收取中..."
      Notice.send(WeChatActions.connectionStatus(), 'syncing');
    } else if (event == 'OnSyncCompleted') {
      // 增量同步完成 → UI 恢复正常
      Notice.send(WeChatActions.connectionStatus(), 'connected');
    }
  }

  /// 主动登出：清空所有 Token 并跳转登录页。
  static Future<void> logout() async {
    await _forceLogout();
  }

  /// 强制清理 Token 并跳转登录页（内部公共路径）。
  static Future<void> _forceLogout() async {
    _wsStarted = false;
    // 双向清空：先清 Rust 内存，再清 Flutter 磁盘
    await rust_api.coreUpdateTokens(
      accessToken: '',
      refreshToken: '',
      accessExpiresAt: null,
      refreshExpiresAt: null,
      userId: null,
    );
    await SharedUtil.instance.saveBoolean(Keys.hasLogged, false);
    await SharedUtil.instance.saveString('access_token', '');
    await SharedUtil.instance.saveString('refresh_token', '');
    await SharedUtil.instance.saveInt('access_expires_at', 0);
    await SharedUtil.instance.saveInt('refresh_expires_at', 0);
    Get.offAll(() => LoginBeginPage());
  }
}
