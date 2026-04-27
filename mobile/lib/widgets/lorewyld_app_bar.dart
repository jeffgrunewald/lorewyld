import 'package:flutter/material.dart';

class LorewyldAppBar extends StatelessWidget implements PreferredSizeWidget {
  const LorewyldAppBar({super.key, this.leading});

  final Widget? leading;

  static const Color _backgroundColor = Color(0xFF6DAE72);
  static const double _wordmarkHeight = 46;

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  Widget build(BuildContext context) {
    return AppBar(
      backgroundColor: _backgroundColor,
      automaticallyImplyLeading: false,
      leading: leading,
      title: Image.asset(
        'assets/branding/wordmark.png',
        height: _wordmarkHeight,
        fit: BoxFit.contain,
        semanticLabel: 'Lorewyld',
      ),
    );
  }
}
