import 'package:extended_text_library/extended_text_library.dart';
import 'package:flutter/material.dart';

import 'at_text.dart';
import 'emoji_text.dart';
import 'link_text.dart';

class TextSpanBuilder extends SpecialTextSpanBuilder {
  final bool showAtBackground;

  TextSpanBuilder({
    this.showAtBackground = false,
  });

  @override
  TextSpan build(String data,
      {TextStyle? textStyle, SpecialTextGestureTapCallback? onTap}) {
    final TextSpan result =
        super.build(data, textStyle: textStyle, onTap: onTap);
    return result;
  }

  @override
  SpecialText? createSpecialText(String flag,
      {TextStyle? textStyle,
      SpecialTextGestureTapCallback? onTap,
      int? index}) {
    if (flag.isEmpty) {
      return null;
    }

    if (isStart(flag, EmojiText.flag)) {
      return EmojiText(textStyle!, start: index! - (EmojiText.flag.length - 1));
    } else if (isStart(flag, AtText.flag)) {
      return AtText(textStyle!, onTap,
          start: index! - (AtText.flag.length - 1),
          showAtBackground: showAtBackground);
    } else if (isStart(flag, LinkText.flag)) {
      return LinkText(textStyle!, onTap,
          start: index! - (LinkText.flag.length - 1));
    }
    return null;
  }
}

class SpecialTextStyle {
  TextRange? textRange;
}
