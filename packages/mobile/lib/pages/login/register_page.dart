import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:image_picker/image_picker.dart';
import 'package:provider/provider.dart';

import 'package:flutter/gestures.dart';
import '../../provider/login_model.dart';
import '../../src/rust/api.dart' as rust_api;
import '../../tools/wechat_flutter.dart';
import '../../ui/web/web_view.dart';
import 'select_location_page.dart';

class RegisterPage extends StatefulWidget {
  const RegisterPage({super.key});

  @override
  _RegisterPageState createState() => _RegisterPageState();
}

class _RegisterPageState extends State<RegisterPage> {
  bool isSelect = false;

  FocusNode nickF = FocusNode();
  TextEditingController nickC = TextEditingController();
  FocusNode phoneF = FocusNode();
  TextEditingController phoneC = TextEditingController();
  FocusNode pWF = FocusNode();
  TextEditingController pWC = TextEditingController();

  String localAvatarImgPath = '';

  Future<void> _openGallery() async {
    final XFile? img = await ImagePicker().pickImage(source: ImageSource.gallery);

    if (img != null) {
      localAvatarImgPath = img.path;
      setState(() {});
    } else {
      return;
    }
  }

  bool isPwdVisible = false;

  Widget body(LoginModel model) {
    final List<Widget> column = <Widget>[
      const Padding(
        padding: EdgeInsets.only(
            left: 5.0, top: mainSpace * 3, bottom: mainSpace * 2),
        child: Center(
          child: Text('用手机号注册', style: TextStyle(fontSize: 25.0, fontWeight: FontWeight.normal)),
        )
      ),
      Center(
        child: InkWell(
          child: Container(
            width: 70.0,
            height: 70.0,
            decoration: BoxDecoration(
              color: Colors.grey.withOpacity(0.15),
              borderRadius: BorderRadius.circular(10.0),
            ),
            child: !strNoEmpty(localAvatarImgPath)
                ? const Icon(Icons.person, color: Colors.grey, size: 50.0)
                : ClipRRect(
                    borderRadius: const BorderRadius.all(Radius.circular(10.0)),
                    child: Image.file(File(localAvatarImgPath), width: 70.0, height: 70.0, fit: BoxFit.cover),
                  ),
          ),
          onTap: () => _openGallery(),
        )
      ),
      const SizedBox(height: 30),
      Container(
        padding: const EdgeInsets.symmetric(vertical: 5.0),
        decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
        child: Row(
          children: <Widget>[
            Container(
              width: Get.width * 0.25,
              alignment: Alignment.centerLeft,
              margin: const EdgeInsets.only(left: 25.0),
              child: const Text(
                '昵称',
                style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
              ),
            ),
            Expanded(
              child: TextField(
                controller: nickC,
                decoration: InputDecoration(
                    hintText: '请填写昵称',
                    hintStyle: TextStyle(color: Colors.grey.withOpacity(0.8)),
                    border: InputBorder.none),
                onChanged: (String text) => setState(() {}),
              )
            )
          ],
        ),
      ),
      Container(
        padding: const EdgeInsets.symmetric(vertical: 15.0),
        decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
        child: Row(
          children: <Widget>[
            Container(
              width: Get.width * 0.25,
              alignment: Alignment.centerLeft,
              margin: const EdgeInsets.only(left: 25.0),
              child: const Text('国家/地区',
                  style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400)),
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
                      model.area,
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
        padding: const EdgeInsets.only(bottom: 5.0),
        decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
        child: Row(
          children: <Widget>[
            Container(
              width: Get.width * 0.25,
              alignment: Alignment.centerLeft,
              margin: const EdgeInsets.only(left: 25.0),
              child: const Text(
                '手机号',
                style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
              ),
            ),
            const Text('+86 ', style: TextStyle(fontSize: 16.0, color: Colors.grey)),
            Expanded(
              child: TextField(
                controller: phoneC,
                keyboardType: TextInputType.phone,
                decoration: InputDecoration(
                    hintText: '请填写手机号',
                    hintStyle: TextStyle(color: Colors.grey.withOpacity(0.8)),
                    border: InputBorder.none),
                onChanged: (String text) => setState(() {}),
              )
            )
          ],
        ),
      ),
      Container(
        padding: const EdgeInsets.only(bottom: 5.0),
        decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2), width: 0.5))),
        child: Row(
          children: <Widget>[
            Container(
              width: Get.width * 0.25,
              alignment: Alignment.centerLeft,
              margin: const EdgeInsets.only(left: 25.0),
              child: const Text(
                '密码',
                style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w400),
              ),
            ),
            Expanded(
              child: TextField(
                controller: pWC,
                obscureText: !isPwdVisible,
                decoration: InputDecoration(
                    hintText: '请设置密码',
                    hintStyle: TextStyle(color: Colors.grey.withOpacity(0.8)),
                    border: InputBorder.none),
                onChanged: (String text) => setState(() {}),
              )
            ),
            InkWell(
              onTap: () => setState(() => isPwdVisible = !isPwdVisible),
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 10),
                child: Icon(isPwdVisible ? Icons.visibility : Icons.visibility_off, color: Colors.grey),
              ),
            )
          ],
        ),
      ),
      const SizedBox(height: mainSpace * 3),
      Row(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          InkWell(
            onTap: () => setState(() => isSelect = !isSelect),
            child: Padding(
              padding: const EdgeInsets.only(top: 2, right: 8),
              child: Icon(
                isSelect ? Icons.check_circle : Icons.radio_button_unchecked,
                color: isSelect ? const Color.fromRGBO(8, 191, 98, 1.0) : Colors.grey.withOpacity(0.6),
                size: 22,
              )
            ),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              RichText(
                text: TextSpan(
                  style: const TextStyle(color: Colors.grey, fontSize: 13),
                  children: <InlineSpan>[
                    const TextSpan(text: '我已阅读并同意'),
                    TextSpan(
                      text: '《软件许可及服务协议》',
                      style: TextStyle(color: Colors.blue[800]),
                      recognizer: TapGestureRecognizer()
                        ..onTap = () {
                          Get.to<void>(WebViewPage(url: S.of(context).protocolUrl, title: S.of(context).protocolTitle));
                        },
                    ),
                  ]
                )
              ),
              InkWell(
                onTap: () => Get.to<void>(WebViewPage(url: S.of(context).protocolUrl, title: S.of(context).protocolTitle)),
                child: Container(
                  padding: const EdgeInsets.only(top: 2, bottom: 2), // Make touch target larger
                  child: Text('本页面收集的信息仅用于注册账号', style: const TextStyle(color: Colors.grey, fontSize: 13))
                ),
              )
            ],
          )
        ],
      ),
      Center(
        child: ComMomButton(
          text: '同意并继续',
          width: Get.width * 0.6,
          height: 48.0,
        style: TextStyle(
            color: pWC.text == '' || phoneC.text == '' || nickC.text == '' || !isSelect ? Colors.grey.withOpacity(0.8) : Colors.white,
            fontSize: 16.0,
            fontWeight: FontWeight.w500),
        margin: const EdgeInsets.only(top: 30.0),
        color: pWC.text == '' || phoneC.text == '' || nickC.text == '' || !isSelect
            ? const Color.fromRGBO(226, 226, 226, 1.0)
            : const Color.fromRGBO(8, 191, 98, 1.0),
        onTap: () async {
          if (pWC.text == '' || phoneC.text == '' || nickC.text == '') return;
          if (!isSelect) return;
          
          if (!GetUtils.isPhoneNumber(phoneC.text)) {
            showToast('请输入正确的手机号');
            return;
          }
          
          try {
            await rust_api.coreRegister(mobile: phoneC.text, password: pWC.text, nickname: nickC.text);
            showToast('注册成功, 请登录');
            Get.back();
          } catch (e) {
            showToast('注册失败: \n$e');
          }
        },
      )),
    ];

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20.0),
      child: Column(
          crossAxisAlignment: CrossAxisAlignment.start, children: column),
    );
  }

  @override
  Widget build(BuildContext context) {
    final LoginModel model = Provider.of<LoginModel>(context);

    return Scaffold(
      appBar:
          const ComMomBar(leadingImg: 'assets/images/bar_close.png'),
      body: MainInputBody(
        color: appBarColor,
        child: SingleChildScrollView(child: body(model)),
        onTap: () => setState(() => <dynamic, dynamic>{}),
      ),
    );
  }
}
