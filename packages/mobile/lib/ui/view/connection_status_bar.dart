import 'package:flutter/material.dart';

/// 微信风格的连接状态指示条
///
/// 在聊天列表顶部显示 WS 连接状态：
/// - "连接中..." — 正在连接/重连（蓝色呼吸动画）
/// - "收取中..." — 增量同步进行中（蓝色加载动画）
/// - 正常连接 — 自动隐藏
class ConnectionStatusBar extends StatefulWidget {
  final String status; // "connected" | "connecting" | "reconnecting" | "syncing"

  const ConnectionStatusBar({super.key, required this.status});

  @override
  State<ConnectionStatusBar> createState() => _ConnectionStatusBarState();
}

class _ConnectionStatusBarState extends State<ConnectionStatusBar>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _opacity;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 800),
    )..repeat(reverse: true);
    _opacity = Tween<double>(begin: 0.6, end: 1.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.status == 'connected') {
      return const SizedBox.shrink();
    }

    String text;
    IconData icon;

    switch (widget.status) {
      case 'syncing':
        text = '收取中...';
        icon = Icons.sync;
        break;
      case 'connecting':
        text = '连接中...';
        icon = Icons.cloud_off_outlined;
        break;
      case 'reconnecting':
        text = '连接中...';
        icon = Icons.cloud_off_outlined;
        break;
      default:
        return const SizedBox.shrink();
    }

    return FadeTransition(
      opacity: _opacity,
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(vertical: 6.0, horizontal: 16.0),
        color: const Color(0xFFF0F3FF),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            SizedBox(
              width: 14,
              height: 14,
              child: widget.status == 'syncing'
                  ? const CircularProgressIndicator(
                      strokeWidth: 2.0,
                      valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF576B95)),
                    )
                  : Icon(icon, size: 14, color: const Color(0xFF576B95)),
            ),
            const SizedBox(width: 6),
            Text(
              text,
              style: const TextStyle(
                fontSize: 13,
                color: Color(0xFF576B95),
                fontWeight: FontWeight.w400,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
