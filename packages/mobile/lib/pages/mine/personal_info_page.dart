import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:image_picker/image_picker.dart';
import 'package:provider/provider.dart';
import 'package:wechat_flutter/pages/mine/change_name_page.dart';
import 'package:wechat_flutter/pages/mine/code_page.dart';
import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/orther/label_row.dart';

class PersonalInfoPage extends StatefulWidget {
  @override
  _PersonalInfoPageState createState() => _PersonalInfoPageState();
}

class _PersonalInfoPageState extends State<PersonalInfoPage> {
  @override
  void initState() {
    super.initState();
  }

  action(v) {
    if (v == '二维码名片') {
      Get.to<void>(new CodePage());
    } else {
      print(v);
    }
  }

  _openGallery({type = ImageSource.gallery}) async {
    showToast('服务器正在调整，敬请期待');

    // final model = Provider.of<GlobalModel>(context, listen: false);
    // File imageFile = await ImagePicker.pickImage(source: type);
    // List<int> imageBytes = await compressFile(imageFile);
    // if (imageFile != null) {
    //   String base64Img = 'data:image/jpeg;base64,${base64Encode(imageBytes)}';
    //   uploadImgApi(context, base64Img, (v) {
    //     if (v == null) {
    //       showToast( '上传头像失败,请换张图像再试');
    //       return;
    //     }
    //
    //     setUsersProfileMethod(
    //       context,
    //       avatarStr: v,
    //       nickNameStr: model.nickName,
    //       callback: (data) {
    //         if (data.toString().contains('ucc')) {
    //           showToast( '设置头像成功');
    //           model.avatar = v;
    //           model.refresh();
    //         } else {
    //           showToast( '设置头像失败');
    //         }
    //       },
    //     );
    //   });
    // }
  }

  Widget dynamicAvatar(String avatar, {double? size}) {
    if (isNetWorkImg(avatar)) {
      return new CachedNetworkImage(
          imageUrl: avatar,
          cacheManager: cacheManager,
          width: size ?? null,
          height: size ?? null,
          fit: BoxFit.fill);
    } else {
      return new Image.asset(avatar,
          fit: BoxFit.fill, width: size ?? null, height: size ?? null);
    }
  }

  Widget body(GlobalModel model) {
    List<Map<String, dynamic>> data = [
      {'label': '名字', 'value': model.nickName ?? model.account},
      {'label': '性别', 'value': '男'},
      {'label': '地区', 'value': '北京 昌平'},
      {'label': '手机号', 'value': '138******34'},
      {'label': '微信号', 'value': 'admin'},
      {'label': '我的二维码', 'value': ''},
      {'label': '拍一拍', 'value': ''},
      {'label': '签名', 'value': '幸福归来。我走了那么远的路，百转千回，只为与你相逢。', 'isLine': false, 'gap': true},
      {'label': '来电铃声', 'value': '就是爱你', 'isLine': false, 'gap': true},
      {'label': '我的地址', 'value': ''},
      {'label': '我的发票抬头', 'value': '', 'isLine': false, 'gap': true},
      {'label': '微信豆', 'value': '', 'isLine': false},
    ];

    var content = [
      new LabelRow(
        label: '头像',
        isLine: true,
        isRight: true,
        rightW: new SizedBox(
          width: 28.0,
          height: 28.0,
          child: new ClipRRect(
            borderRadius: BorderRadius.all(Radius.circular(4.0)),
            child: strNoEmpty(model.avatar)
                ? dynamicAvatar(model.avatar)
                : new Image.asset(defIcon, fit: BoxFit.cover),
          ),
        ),
        onPressed: () => _openGallery(),
      ),
      new Column(
        children: data.map((item) => buildContent(item, model)).toList(),
      ),
    ];

    return new Column(children: content);
  }

  Widget buildContent(Map<String, dynamic> item, GlobalModel model) {
    bool isGap = item['gap'] == true;
    bool isLine = item['isLine'] ?? true;
    String label = item['label'];

    return new LabelRow(
      label: label,
      rValue: item['value'],
      isLine: isLine,
      isRight: true,
      isTopAlign: label == '签名', // Ensure long multiline text aligns its Top label with the text block
      margin: EdgeInsets.only(bottom: isGap ? 8.0 : 0.0), // Standard grouping gap
      rightW: label == '我的二维码'
          ? new Container(
              margin: EdgeInsets.only(right: 4.0),
              child: new Image.asset('assets/images/mine/ic_small_code.png',
                  color: Color(0xFFC7C7CC), width: 14.0),
            )
          : null,
      onPressed: () {
        if (label == '名字') {
          Get.to<void>(new ChangeNamePage(model.nickName));
        } else {
          action(label);
        }
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final model = Provider.of<GlobalModel>(context);

    return new Scaffold(
      backgroundColor: appBarColor,
      appBar: new ComMomBar(title: '个人资料', centerTitle: true),
      body: new SingleChildScrollView(child: body(model)),
    );
  }
}
