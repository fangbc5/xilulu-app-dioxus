import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';

class LabelRow extends StatelessWidget {
  final String? label;
  final VoidCallback? onPressed;
  final double? labelWidth;
  final bool isRight;
  final bool isLine;
  final String? value;
  final String? rValue;
  final Widget? rightW;
  final EdgeInsetsGeometry? margin;
  final EdgeInsetsGeometry padding;
  final Widget? headW;
  final double lineWidth;

  LabelRow({
    this.label,
    this.onPressed,
    this.value,
    this.labelWidth,
    this.isRight = true,
    this.isLine = false,
    this.rightW,
    this.rValue,
    this.margin,
    this.padding = const EdgeInsets.only(top: 14.0, bottom: 14.0, right: 15.0),
    this.headW,
    this.lineWidth = mainLineWidth,
    this.isTopAlign = false,
  });

  final bool isTopAlign;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: margin,
      child: TextButton(
        style: TextButton.styleFrom(
          backgroundColor: Colors.white,
          padding: EdgeInsets.all(0),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        ),
        onPressed: onPressed ?? () {},
        child: Container(
          padding: padding,
          margin: EdgeInsets.only(left: 20.0),
          decoration: BoxDecoration(
            border: isLine
                ? Border(bottom: BorderSide(color: Color(0xFFEFEFEF), width: 0.5)) // Faint standard iOS gray line
                : null,
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: <Widget>[
              if (headW != null) headW!,
              Expanded(
                child: Row(
                  crossAxisAlignment: isTopAlign ? CrossAxisAlignment.start : CrossAxisAlignment.center,
                  children: [
                    SizedBox(
                      width: labelWidth,
                      child: Text(
                        label ?? '',
                        style: TextStyle(color: Color(0xFF333333), fontSize: 16.0, fontWeight: FontWeight.w400),
                      ),
                    ),
                    if (value != null)
                      Text(
                        value!,
                        style: TextStyle(
                          color: mainTextColor.withOpacity(0.7),
                        ),
                      ),
                    Spacer(),
                    if (rValue != null)
                      Expanded(
                        child: Text(
                          rValue!,
                          textAlign: TextAlign.right,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: Color(0xFF888888),
                            fontWeight: FontWeight.w400,
                            fontSize: 16.0,
                            height: 1.3, // add line height for the multiline text so it matches original better
                          ),
                        ),
                      ),
                    if (rightW != null) Container(margin: EdgeInsets.only(left: 10.0), child: rightW!),
                  ],
                ),
              ),
              if (isRight)
                Container(
                  width: 8.0,
                  margin: EdgeInsets.only(left: 10.0),
                  child: Image.asset(
                    'assets/images/ic_right_arrow_grey.webp',
                    color: Color(0xFFC7C7CC),
                    fit: BoxFit.cover,
                  ),
                )
              else
                Container(width: 10.0),
            ],
          ),
        ),
      ),
    );
  }
}