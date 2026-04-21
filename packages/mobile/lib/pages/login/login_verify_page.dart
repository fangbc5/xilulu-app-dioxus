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

class LoginVerifyPage extends StatefulWidget {
  
  const LoginVerifyPage({super.key, required this.mobile, this.areaCode = '+86'});
  final String mobile;
  final String areaCode;

  @override
  _LoginVerifyPageState createState() => _LoginVerifyPageState();
}

class _LoginVerifyPageState extends State<LoginVerifyPage> {
  final TextEditingController _tC = TextEditingController();
  bool isPasswordMode = true;
  bool syncChatHistory = true;

  Future<void> doLogin() async {
    if (_tC.text.isEmpty) {
      showToast(isPasswordMode ? '请输入密码' : '请输入验证码');
      return;
    }
    
    // 显示 loading 或弹窗
    try {
      String resp;
      if (isPasswordMode) {
        resp = await rust_api.coreLoginWithPwd(account: widget.mobile, password: _tC.text, region: widget.areaCode);
      } else {
        resp = await rust_api.coreLoginOrRegisterByCode(mobile: widget.mobile, code: _tC.text, region: widget.areaCode);
      }
      
      // 解析出来的 JSON
      final Map<String, dynamic> data = json.decode(resp) as Map<String, dynamic>;
      
      // Attempt to extract login_info (if LoginOrRegisterResponse) or user_info directly (if LoginResponse)
      Map<String, dynamic>? userInfo;
      if (data.containsKey('login_info')) {
        var loginInfo = data['login_info'] as Map<String, dynamic>;
        userInfo = loginInfo['user_info'] as Map<String, dynamic>?;
        if (loginInfo.containsKey('access_token')) {
          data['access_token'] = loginInfo['access_token'];
        }
        if (loginInfo.containsKey('refresh_token')) {
          data['refresh_token'] = loginInfo['refresh_token'];
        }
      } else {
        userInfo = data['user_info'] as Map<String, dynamic>?;
      }
      
      String realAccount = widget.mobile;
      String? nickName;
      String? avatar;
      if (userInfo != null) {
        realAccount = userInfo['id']?.toString() ?? widget.mobile;
        nickName = userInfo['nickname']?.toString();
        avatar = userInfo['avatar']?.toString();
        await SharedUtil.instance.saveString(Keys.account, realAccount);
        if (nickName != null) {
          await SharedUtil.instance.saveString(Keys.nickName, nickName);
        }
        if (avatar != null) {
          await SharedUtil.instance.saveString(Keys.faceUrl, avatar);
        }
      } else {
        await SharedUtil.instance.saveString(Keys.account, widget.mobile);
      }
      
      try {
        final model = Provider.of<GlobalModel>(context, listen: false);
        model.account = realAccount;
        if (nickName != null) model.nickName = nickName;
        if (avatar != null) model.avatar = avatar;
        model.refresh();
      } catch (e) {
        debugPrint('GlobalModel refresh failed: $e');
      }

      if (data.containsKey('access_token')) {
        await SharedUtil.instance.saveString('access_token', data['access_token'] as String);
      }
      if (data.containsKey('refresh_token')) {
        await SharedUtil.instance.saveString('refresh_token', data['refresh_token'] as String);
      }
      
      await SharedUtil.instance.saveBoolean('sync_chat_history', syncChatHistory);
      
      await ImLoginManager.login(realAccount, context);
      // but in case it's decoupled:
      showToast('登录成功');
      Get.offAll(const RootPage());
    } catch (e) {
      showToast('登录失败: \n$e');
    }
  }

  Future<void> getVerifyCode() async {
    try {
      await rust_api.coreSendVerifyCode(mobile: widget.mobile);
      showToast('验证码已发送');
    } catch (e) {
      showToast('发送失败: \n$e');
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
    final List<String> btItem = <String>['找回密码', '冻结账号', '更多'];

    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        leading: InkWell(
          child: const Icon(Icons.arrow_back_ios, color: Colors.black, size: 20),
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
                  padding: EdgeInsets.only(
                      left: 20.0, top: mainSpace * 3, bottom: mainSpace * 2),
                  child: Center(
                    child: Text('手机号登录', style: TextStyle(fontSize: 25.0, fontWeight: FontWeight.normal)),
                  ),
                ),
                const SizedBox(height: 20),
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
                          '手机号',
                          style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
                        ),
                      ),
                      Text('${widget.areaCode} ', style: const TextStyle(fontSize: 16.0, color: Colors.grey)),
                      Padding(
                        padding: const EdgeInsets.only(left: 10.0),
                        child: Text(
                          '${widget.mobile}',
                          style: const TextStyle(fontSize: 16.0),
                        ),
                      ),
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
                        child: Text(
                          isPasswordMode ? '密码' : '验证码',
                          style: const TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
                        ),
                      ),
                      Expanded(
                        child: TextField(
                          controller: _tC,
                          obscureText: isPasswordMode,
                          style: const TextStyle(textBaseline: TextBaseline.alphabetic),
                          decoration: InputDecoration(
                              isDense: true,
                              contentPadding: EdgeInsets.zero,
                              hintText: isPasswordMode ? '请填写密码' : '收到的验证码',
                              hintStyle: TextStyle(color: Colors.grey.withOpacity(0.8)),
                              border: InputBorder.none),
                          onChanged: (String text) {
                            setState(() {});
                          },
                        )
                      ),
                      if (!isPasswordMode)
                        InkWell(
                          onTap: getVerifyCode,
                          child: const Padding(
                            padding: EdgeInsets.only(right: 20),
                            child: Text('获取验证码', style: TextStyle(color: Colors.blue)),
                          ),
                        )
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(left: 25.0, top: 15.0, bottom: 5.0),
                  child: InkWell(
                    child: Text(
                      isPasswordMode ? '用短信验证码登录' : '用密码登录',
                      style: TextStyle(color: Colors.blue[800], fontSize: 13),
                    ),
                    onTap: () {
                      setState(() {
                         isPasswordMode = !isPasswordMode;
                         _tC.clear();
                      });
                    },
                  ),
                ),
              ],
            ),
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
                    text: '登录',
                    width: Get.width * 0.45,
                    height: 48.0,
                    style: TextStyle(
                        color: _tC.text == '' ? Colors.white.withOpacity(0.8) : Colors.white,
                        fontSize: 16.0,
                        fontWeight: FontWeight.w500),
                    margin: const EdgeInsets.only(top: 20.0, bottom: 40.0),
                    color: _tC.text == ''
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
