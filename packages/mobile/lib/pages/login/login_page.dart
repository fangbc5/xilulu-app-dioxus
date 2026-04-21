import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:provider/provider.dart';

import '../../provider/login_model.dart';
import '../../tools/wechat_flutter.dart';
import 'login_email_pwd_page.dart';
import 'login_verify_page.dart';
import 'select_location_page.dart';

class LoginPage extends StatefulWidget {
  const LoginPage({super.key});

  @override
  _LoginPageState createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  final TextEditingController _tC = TextEditingController();
  bool isSelect = false;

  @override
  void initState() {
    super.initState();
    initEdit();
  }

  Future<void> initEdit() async {
    final String? user = await SharedUtil.instance.getString(Keys.account);
    _tC.text = user ?? '';
  }

  Widget bottomItem(String item) {
    return Row(
      children: <Widget>[
        InkWell(
          child: Text(item, style: const TextStyle(color: tipColor)),
          onTap: () {
            showToast( S.of(context).notOpen + item);
          },
        ),
        if (item == '更多') Container() else Padding(
                padding: const EdgeInsets.symmetric(horizontal: 5.0),
                child: VerticalLine(height: 15.0),
              )
      ],
    );
  }

  Widget body(LoginModel model) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        const Padding(
          padding: EdgeInsets.only(top: 80.0, bottom: 40.0),
          child: Center(
            child: Text('手机号登录', style: TextStyle(fontSize: 26.0, fontWeight: FontWeight.w500)),
          )
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
                child: Text(S.of(context).phoneCity,
                    style: const TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400)),
              ),
              Expanded(
                child: InkWell(
                  onTap: () async {
                    final result = await Get.to<String?>(new SelectLocationPage());
                    if (result == null) return;
                    model.area = result;
                    model.refresh();
                    SharedUtil.instance.saveString(Keys.area, result);
                  },
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: <Widget>[
                      Text(
                        model.area.replaceAll(RegExp(r'\s*\(.*\)'), ''), // Strip (+86)
                        style: const TextStyle(color: Colors.black, fontSize: 16.0, fontWeight: FontWeight.w400),
                      ),
                      const Icon(Icons.chevron_right, color: Colors.grey)
                    ],
                  ),
                ),
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
                child: Text(
                  S.of(context).phoneNumber,
                  style: const TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
                ),
              ),
              Text(
                '${RegExp(r'\((.*?)\)').firstMatch(model.area)?.group(1) ?? '+86'} ',
                style: const TextStyle(fontSize: 16.0, color: Colors.grey),
              ),
              Expanded(
                  child: TextField(
                controller: _tC,
                style: const TextStyle(textBaseline: TextBaseline.alphabetic, fontSize: 16.0),
                keyboardType: TextInputType.phone,
                inputFormatters: <TextInputFormatter>[
                  FilteringTextInputFormatter(RegExp(r'[0-9]'), allow: true)
                ],
                decoration: InputDecoration(
                    isDense: true,
                    contentPadding: EdgeInsets.zero,
                    hintText: '请填写手机号码',
                    hintStyle: TextStyle(color: Colors.grey.withOpacity(0.5), fontSize: 16.0),
                    border: InputBorder.none),
                onChanged: (String text) {
                  setState(() {});
                },
              ))
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.only(left: 25.0, top: 15.0, bottom: 5.0),
          child: Text(
            '上述手机号仅用于登录验证',
            style: const TextStyle(color: Color.fromRGBO(175, 175, 175, 1.0), fontSize: 12),
          ),
        ),
        Padding(
          padding: const EdgeInsets.only(left: 25.0, bottom: 20.0),
          child: InkWell(
            child: Text(
              '用微信号 /QQ 号 /邮箱登录',
              style: TextStyle(color: Colors.blue[800], fontSize: 14),
            ),
            onTap: () {
               Get.to(() => const LoginEmailPwdPage());
            },
          ),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final LoginModel model = Provider.of<LoginModel>(context);

    final List<String> btItem = <String>[
      '找回密码',
      '导出聊天记录',
      '更多',
    ];

    return Scaffold(
      backgroundColor: Colors.white,
      appBar: const ComMomBar(
          leadingImg: 'assets/images/bar_close.png', 
          backgroundColor: Colors.white),
      body: MainInputBody(
        color: Colors.white,
        child: Stack(
          children: <Widget>[
            SingleChildScrollView(child: body(model)),
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
                        onTap: () => setState(() => isSelect = !isSelect),
                        child: Padding(
                          padding: const EdgeInsets.only(right: 5),
                          child: Icon(
                            isSelect ? Icons.check_circle : Icons.radio_button_unchecked, 
                            color: isSelect ? const Color.fromRGBO(8, 191, 98, 1.0) : Colors.grey, 
                            size: 22
                          ),
                        ),
                      ),
                      const Text('登录后同步最近的聊天记录', style: TextStyle(color: Colors.grey, fontSize: 13))
                    ],
                  ),
                  Center(
                    child: ComMomButton(
                      text: '同意并继续',
                      width: Get.width * 0.45,
                      height: 48.0,
                      style: TextStyle(
                          color: _tC.text == '' ? Colors.white.withOpacity(0.8) : Colors.white,
                          fontSize: 16.0,
                          fontWeight: FontWeight.w500),
                      margin: const EdgeInsets.only(top: 20.0, bottom: 40.0),
                      color: _tC.text == ''
                          ? const Color.fromRGBO(8, 191, 98, 0.5) // Light green when inactive based on screenshot
                          : const Color.fromRGBO(8, 191, 98, 1.0),
                      onTap: () async {
                        if (_tC.text == '') {
                          showToast('请输入手机号');
                        } else if (_tC.text.length >= 3) {
                          final areaCode = RegExp(r'\((.*?)\)').firstMatch(model.area)?.group(1) ?? '+86';
                          await SharedUtil.instance.saveBoolean('sync_chat_history', isSelect);
                          Get.to(() => LoginVerifyPage(mobile: _tC.text, areaCode: areaCode));
                        } else {
                          showToast('请输入正确的手机号');
                        }
                      },
                    ),
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: btItem.map(bottomItem).toList(),
                  ),
                ],
              )
            ),
          ],
        ),
      ),
    );
  }
}
