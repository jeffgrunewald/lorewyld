import 'package:flutter/material.dart';

import 'dice_type.dart';

class DiceIcon extends StatelessWidget {
  const DiceIcon({super.key, required this.type, this.size = 48});

  final DiceType type;
  final double size;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: size,
      height: size,
      child: Image.asset(
        'assets/dice/${type.label}.png',
        fit: BoxFit.contain,
        alignment: type == DiceType.d4
            ? const Alignment(0, -0.5)
            : Alignment.center,
        semanticLabel: type.label,
      ),
    );
  }
}
