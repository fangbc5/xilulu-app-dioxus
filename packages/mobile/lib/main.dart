import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wechat_flutter/config/provider_config.dart';
import 'package:wechat_flutter/app.dart';
import 'package:wechat_flutter/tools/data/data.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:path_provider/path_provider.dart';

import 'config/storage_manager.dart';
import 'package:wechat_flutter/im/login_handle.dart';

import 'src/rust/frb_generated.dart';
import 'src/rust/api/im.dart';

void main() async {
  /// 确保初始化
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  
  // 初始化 Rust 核心 SDK 数据库 (正式落地为本地存储)
  final supportDir = await getApplicationDocumentsDirectory();
  final dbPath = "${supportDir.path}/im_local.db";
  
  debugPrint("=====================================================");
  debugPrint("🟢 [RUST SQLITE DB PATH]: $dbPath");
  debugPrint("=====================================================");
  
  await coreInitSdk(dbPath: dbPath);
  
  // 初始化全局事件监听器 (接收 AUTH_EXPIRED 等)
  await ImLoginManager.initGlobalListener();

  // 冷启动 / hot restart 种子注入与防线：
  // 严格把控只有完全有效的 Token 才会注入底层，否则立拔当前本地缓存令其回落到登录
  await ImLoginManager.checkAndInjectTokens();
  
  /// 数据初始化
  await Data.initData();

  // StorageManager 已在 Token 种子注入时初始化，无需重复调用

  /// APP入口并配置Provider
  runApp(ProviderConfig.getInstance().getGlobal(MyApp()));

  /// 自定义报错页面
  ErrorWidget.builder = (FlutterErrorDetails flutterErrorDetails) {
    debugPrint(flutterErrorDetails.toString());
    return new Center(child: new Text("App错误，快去反馈给作者!"));
  };

  /// Android状态栏透明
  if (Platform.isAndroid) {
    SystemUiOverlayStyle systemUiOverlayStyle =
        SystemUiOverlayStyle(statusBarColor: Colors.transparent);
    SystemChrome.setSystemUIOverlayStyle(systemUiOverlayStyle);
  }
}
