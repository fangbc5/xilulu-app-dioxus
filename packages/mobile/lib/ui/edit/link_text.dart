import 'package:extended_text_library/extended_text_library.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

class LinkText extends SpecialText {
  static const String flag = "http";
  final int start;

  LinkText(TextStyle textStyle, SpecialTextGestureTapCallback? onTap,
      {required this.start})
      : super(flag, " ", textStyle, onTap: onTap);

  @override
  bool isEnd(String value) {
    var end = value.endsWith(" ") || value.endsWith("\n");
    return end;
  }

  @override
  InlineSpan finishText() {
    final String text = toString();
    TextStyle textStyle =
        this.textStyle?.copyWith(color: Colors.blue, decoration: TextDecoration.underline) ??
            const TextStyle(color: Colors.blue, decoration: TextDecoration.underline);

    return SpecialTextSpan(
      text: text,
      actualText: text,
      start: start,
      style: textStyle,
      recognizer: TapGestureRecognizer()
        ..onTap = () {
          if (onTap != null) onTap!(text);
        },
    );
  }
}
