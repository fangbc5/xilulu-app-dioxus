import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'dart:convert';
import '../../im/login_handle.dart';
import '../../src/rust/api/auth.dart' as rust_api;
import '../../tools/wechat_flutter.dart';
import '../root/root_page.dart';
import '../../ui/view/main_input.dart';
import 'package:provider/provider.dart';
import 'package:wechat_flutter/provider/global_model.dart';

class LoginEmailPwdPage extends StatefulWidget {
  const LoginEmailPwdPage({super.key});

  @override
  _LoginEmailPwdPageState createState() => _LoginEmailPwdPageState();
}


class _LoginEmailPwdPageState extends State<LoginEmailPwdPage> {
  final TextEditingController _accountC = TextEditingController();
  final TextEditingController _pwdC = TextEditingController();
  bool syncChatHistory = true;

  Future<void> doLogin() async {
    if (_accountC.text.isEmpty || _pwdC.text.isEmpty) {
      showToast('请输入完整信息');
      return;
    }
    
    try {
      showToast('登录中...');
      final respJson = await rust_api.coreLoginWithPwd(
        account: _accountC.text,
        password: _pwdC.text,
        region: null,
      );
      
      // 解析用户信息（仅用于 UI 更新），Token 已由 Rust auth.rs 写入 GLOBAL_STORAGE
      final Map<String, dynamic> data = json.decode(respJson) as Map<String, dynamic>;
      final userInfo = data['user_info'] as Map<String, dynamic>?;
      final String userId    = userInfo?['id']?.toString() ?? _accountC.text;
      final String? nickName = userInfo?['nickname']?.toString();
      final String? avatar   = userInfo?['avatar']?.toString();

      // 持久化用户 UI 信息 + 标记已登录
      await SharedUtil.instance.saveBoolean(Keys.hasLogged, true);
      await SharedUtil.instance.saveString(Keys.account, userId);
      if (nickName != null) await SharedUtil.instance.saveString(Keys.nickName, nickName);
      if (avatar   != null) await SharedUtil.instance.saveString(Keys.faceUrl, avatar);
      
      // 持久化 Token 到 SharedPreferences（作为冷启动备份，TOKEN_REFRESHED 事件会持续更新）
      final String access  = data['access_token']  as String? ?? '';
      final String refresh = data['refresh_token'] as String? ?? '';
      if (access.isNotEmpty) {
        await SharedUtil.instance.saveString('access_token', access);
        await SharedUtil.instance.saveString('refresh_token', refresh);
      }

      // 更新内存中的 GlobalModel
      try {
        final model = Provider.of<GlobalModel>(context, listen: false);
        model.account = userId;
        if (nickName != null) model.nickName = nickName;
        if (avatar   != null) model.avatar = avatar;
        model.refresh();
      } catch (e) {
        debugPrint('GlobalModel refresh failed: $e');
      }

      // 启动 WS 会话（Token 已在 Rust GLOBAL_STORAGE 中）
      await ImLoginManager.startWs(syncChatHistory: syncChatHistory);
      showToast('登录成功');
      Get.offAll(const RootPage());
    } catch (e) {
      debugPrint('Login Error: $e');
      showToast('登录失败: \n$e');
    }
  }

  Widget bottomItem(String item) {
    return Row(
      children: <Widget>[
        InkWell(
          child: Text(item, style: const TextStyle(color: tipColor)),
          onTap: () {
            showToast('暂未开放: $item');
          },
        ),
        if (item == '更多') Container() else Padding(
                padding: const EdgeInsets.symmetric(horizontal: 5.0),
                child: VerticalLine(height: 15.0),
              )
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final List<String> btItem = <String>['找回密码', '导出聊天记录', '更多'];

    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        leading: InkWell(
          child: const Icon(Icons.close, color: Colors.black, size: 25),
          onTap: () => Get.back(),
        ),
        backgroundColor: Colors.white,
        elevation: 0,
      ),
      body: MainInputBody(
        color: Colors.white,
        child: Stack(
          children: <Widget>[
          SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                const Padding(
                  padding: EdgeInsets.only(top: 80.0, bottom: 40.0),
                  child: Center(
                    child: Text('微信号/QQ号/邮箱登录', style: TextStyle(fontSize: 26.0, fontWeight: FontWeight.w500)),
                  ),
                ),
                Container(
                  height: 56.0,
                  margin: const EdgeInsets.symmetric(horizontal: 25.0),
                  decoration: BoxDecoration(
                      border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: <Widget>[
                      SizedBox(
                        width: Get.width * 0.25,
                        child: const Text(
                          '账号',
                          style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
                        ),
                      ),
                      Expanded(
                        child: TextField(
                          controller: _accountC,
                          style: const TextStyle(fontSize: 16.0),
                          decoration: InputDecoration(
                              isDense: true,
                              contentPadding: EdgeInsets.zero,
                              hintText: '微信号/QQ号/邮箱',
                              hintStyle: TextStyle(color: Colors.grey.withOpacity(0.5), fontSize: 16.0),
                              border: InputBorder.none),
                          onChanged: (String text) {
                            setState(() {});
                          },
                        )
                      )
                    ],
                  ),
                ),
                Container(
                  height: 56.0,
                  margin: const EdgeInsets.symmetric(horizontal: 25.0),
                  decoration: BoxDecoration(
                      border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: <Widget>[
                      SizedBox(
                        width: Get.width * 0.25,
                        child: const Text(
                          '密码',
                          style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
                        ),
                      ),
                      Expanded(
                        child: TextField(
                          controller: _pwdC,
                          style: const TextStyle(textBaseline: TextBaseline.alphabetic, fontSize: 16.0),
                          obscureText: true,
                          decoration: InputDecoration(
                              isDense: true,
                              contentPadding: EdgeInsets.zero,
                              hintText: '请填写密码',
                              hintStyle: TextStyle(color: Colors.grey.withOpacity(0.5), fontSize: 16.0),
                              border: InputBorder.none),
                          onChanged: (String text) {
                            setState(() {});
                          },
                        )
                      )
                    ],
                  ),
                ),
                const Padding(
                  padding: EdgeInsets.only(left: 25.0, top: 15.0, bottom: 5.0),
                  child: Text(
                    '上述微信号/QQ号/邮箱仅用于登录验证',
                    style: TextStyle(color: Color.fromRGBO(175, 175, 175, 1.0), fontSize: 12),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(left: 25.0, bottom: 20.0),
                  child: InkWell(
                    child: Text(
                      '用手机号登录',
                      style: TextStyle(color: Colors.blue[800], fontSize: 14),
                    ),
                    onTap: () {
                       Get.back(); // Returns to Login Page
                    },
                  ),
                ),
              ],
            )
          ),
          Positioned(
            bottom: 80,
            left: 0,
            right: 0,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: <Widget>[
                    InkWell(
                      onTap: () => setState(() => syncChatHistory = !syncChatHistory),
                      child: Padding(
                        padding: const EdgeInsets.only(right: 5),
                        child: Icon(
                          syncChatHistory ? Icons.check_circle : Icons.radio_button_unchecked,
                          color: syncChatHistory ? const Color.fromRGBO(8, 191, 98, 1.0) : Colors.grey,
                          size: 22,
                        ),
                      ),
                    ),
                    const Text('登录后同步最近的聊天记录', style: TextStyle(color: Colors.grey, fontSize: 13))
                  ],
                ),
                Center(
                  child: ComMomButton(
                    text: '同意并登录',
                    width: Get.width * 0.45,
                    height: 48.0,
                    style: TextStyle(
                        color: _accountC.text == '' || _pwdC.text == '' ? Colors.white.withOpacity(0.8) : Colors.white,
                        fontSize: 16.0,
                        fontWeight: FontWeight.w500),
                    margin: const EdgeInsets.only(top: 20.0, bottom: 40.0),
                    color: _accountC.text == '' || _pwdC.text == ''
                        ? const Color.fromRGBO(8, 191, 98, 0.5)
                        : const Color.fromRGBO(8, 191, 98, 1.0),
                    onTap: doLogin,
                  ),
                ),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: btItem.map(bottomItem).toList(),
                ),
              ],
            ),
          ),
        ],
      ),
      )
    );
  }
}
