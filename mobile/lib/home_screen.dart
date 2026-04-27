import 'package:flutter/material.dart';

import 'dice/dice_icon.dart';
import 'dice/dice_roller_screen.dart';
import 'dice/dice_type.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  void _openDiceRoller(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const DiceRollerScreen()),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Image.asset(
          'assets/branding/wordmark.png',
          height: 120,
          fit: BoxFit.contain,
          semanticLabel: 'Lorewyld',
        ),
      ),
      floatingActionButton: _D20FloatingButton(
        onPressed: () => _openDiceRoller(context),
      ),
    );
  }
}

class _D20FloatingButton extends StatelessWidget {
  const _D20FloatingButton({required this.onPressed});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: 'Open dice roller',
      child: GestureDetector(
        onTap: onPressed,
        behavior: HitTestBehavior.opaque,
        child: const SizedBox(
          width: 72,
          height: 72,
          child: DiceIcon(type: DiceType.d20, size: 72),
        ),
      ),
    );
  }
}
