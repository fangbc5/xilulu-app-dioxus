import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';

class ImageView extends StatelessWidget {
  final String img;
  final double? width;
  final double? height;
  final BoxFit? fit;
  final bool isRadius;

  ImageView({
    required this.img,
    this.height,
    this.width,
    this.fit,
    this.isRadius = true,
  });

  @override
  Widget build(BuildContext context) {
    Widget _buildDefIcon() {
      return Container(
        decoration: BoxDecoration(
          color: Colors.black26.withOpacity(0.1),
          border: Border.all(color: Colors.black.withOpacity(0.2), width: 0.3),
        ),
        child: Image.asset(
          defIcon,
          width: width != null ? width! - 1 : null,
          height: height != null ? height! - 1 : null,
          fit: width != null && height != null ? BoxFit.fill : fit,
        ),
      );
    }

    Widget image;
    if (img.startsWith('assets/')) {
      image = Image.asset(
        img,
        width: width,
        height: height,
        fit: width != null && height != null ? BoxFit.fill : fit,
      );
    } else if (img.startsWith('http://') || img.startsWith('https://')) {
      image = CachedNetworkImage(
        imageUrl: img,
        width: width,
        height: height,
        fit: fit,
        cacheManager: cacheManager,
        memCacheWidth: width != null ? (width! * 3).toInt() : null,
        memCacheHeight: height != null ? (height! * 3).toInt() : null,
      );
    } else if (img.startsWith('/') || img.startsWith('file://') || img.startsWith(r'C:\')) {
      image = Image.file(
        File(img),
        width: width,
        height: height,
        fit: fit,
        errorBuilder: (context, error, stackTrace) => _buildDefIcon(),
      );
    } else {
      image = _buildDefIcon();
    }
    if (isRadius) {
      return ClipRRect(
        borderRadius: BorderRadius.all(
          Radius.circular(4.0),
        ),
        child: image,
      );
    }
    return image;
  }
}
