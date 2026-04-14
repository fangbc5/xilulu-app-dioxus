import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wechat_flutter/config/provider_config.dart';
import 'package:wechat_flutter/app.dart';
import 'package:wechat_flutter/tools/data/data.dart';

import 'config/storage_manager.dart';

import 'src/rust/frb_generated.dart';
import 'src/rust/api/im.dart';

void main() async {
  /// 确保初始化
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  
  // 初始化 Rust 核心 SDK 数据库 (目前使用内存库方便测试，正式环境后续接入本地存储)
  await coreInitSdk(dbPath: 'sqlite::memory:');
  
  /// 数据初始化
  await Data.initData();

  /// 配置初始化
  await StorageManager.init();

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
