import 'package:flutter/material.dart';

import 'package:wechat_flutter/im/info_handle.dart';
import 'package:wechat_flutter/provider/loginc/global_loginc.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class GlobalModel extends ChangeNotifier {
  BuildContext? context;

  ///app的名字
  String appName = '微信flutter';

  /// 用户信息
  String account = '';
  String nickName = 'nickName';
  String avatar = '';
  int gender = 0;

  ///当前语言
  List<String> currentLanguageCode = ['zh', 'CN'];
  String currentLanguage = '中文';
  Locale? currentLocale;

  ///是否进入登录页
  bool goToLogin = true;

  late GlobalLogic logic;

  GlobalModel() {
    this.logic = GlobalLogic(this);
  }

  void setContext(BuildContext context) {
    if (this.context == null) {
      this.context = context;
      Future.wait([
        logic.getAppName(),
        logic.getCurrentLanguageCode(),
        logic.getCurrentLanguage(),
        logic.getLoginState(),
        logic.getAccount(),
        logic.getNickName(),
        logic.getFaceUrl(),
        logic.getGender(),
      ]).then((value) {
        currentLocale = Locale(currentLanguageCode[0], currentLanguageCode[1]);
        refresh();
      });
    }
  }

  Future<void> initInfo() async {
    final List<XUserInfo> data = await getUsersProfile([account]);
    if (data.isEmpty) {
      return;
    }

    final XUserInfo model = data[0];
    nickName = model.nickName ?? model.userId;

    await SharedUtil.instance.saveString(Keys.nickName, nickName);
    avatar = model.faceUrl ?? '';
    await SharedUtil.instance.saveString(Keys.faceUrl, avatar);
    gender = model.gender ?? 0;
    await SharedUtil.instance.saveInt(Keys.gender, model.gender ?? 0);
  }

  @override
  void dispose() {
    super.dispose();
    debugPrint('GlobalModel销毁了');
  }

  void refresh() {
    if (!goToLogin) {
      initInfo();
    }
    notifyListeners();
  }
}