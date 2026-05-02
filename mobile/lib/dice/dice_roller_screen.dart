import 'dart:math';

import 'package:flutter/material.dart';

import 'dice_icon.dart';
import 'dice_type.dart';

class DiceRollerScreen extends StatefulWidget {
  const DiceRollerScreen({super.key});

  @override
  State<DiceRollerScreen> createState() => _DiceRollerScreenState();
}

class _DiceRollerScreenState extends State<DiceRollerScreen> {
  static const int _maxPerDie = 99;

  final Map<DiceType, int> _queue = {};
  final Map<DiceType, List<int>> _lastRolls = {};
  final Random _random = Random.secure();
  bool _rolled = false;

  void _enqueue(DiceType type) {
    if (_rolled) return;
    setState(() {
      final next = (_queue[type] ?? 0) + 1;
      _queue[type] = next.clamp(1, _maxPerDie);
    });
  }

  void _roll() {
    if (_rolled || _queue.isEmpty) return;
    setState(() {
      _lastRolls
        ..clear()
        ..addEntries(
          _queue.entries.map(
            (e) => MapEntry(
              e.key,
              List.generate(e.value, (_) => e.key.roll(_random)),
            ),
          ),
        );
      _rolled = true;
    });
  }

  void _clear() {
    setState(() {
      _queue.clear();
      _lastRolls.clear();
      _rolled = false;
    });
  }

  int _subtotal(DiceType t) =>
      (_lastRolls[t] ?? const []).fold(0, (a, b) => a + b);

  int get _grandTotal => _lastRolls.values.fold(
        0,
        (a, list) => a + list.fold(0, (p, n) => p + n),
      );

  @override
  Widget build(BuildContext context) {
    final queuedInOrder = DiceType.displayOrder
        .where((t) => (_queue[t] ?? 0) > 0)
        .toList(growable: false);

    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: queuedInOrder.isEmpty
                  ? _buildEmptyHint(context)
                  : ListView.builder(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 12,
                      ),
                      itemCount: queuedInOrder.length,
                      itemBuilder: (context, i) {
                        final t = queuedInOrder[i];
                        return _QueueRow(
                          type: t,
                          count: _queue[t] ?? 0,
                          subtotal: _rolled ? _subtotal(t) : null,
                        );
                      },
                    ),
            ),
            if (_rolled)
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                child: Text(
                  'Total: $_grandTotal',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ),
            const Divider(height: 1),
            _buildControls(context),
          ],
        ),
      ),
    );
  }

  Widget _buildEmptyHint(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Text(
          'Tap a die below to enqueue rolls.',
          style: Theme.of(context).textTheme.bodyLarge,
          textAlign: TextAlign.center,
        ),
      ),
    );
  }

  Widget _buildControls(BuildContext context) {
    const topRow = <DiceType>[
      DiceType.d4,
      DiceType.d6,
      DiceType.d8,
      DiceType.d10,
    ];
    const bottomRow = <DiceType>[
      DiceType.d10Percentile,
      DiceType.d12,
      DiceType.d20,
    ];

    final rollEnabled = _rolled || _queue.isNotEmpty;

    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 12, 12, 16),
      child: IntrinsicHeight(
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: Column(
                children: [
                  _buildDiceRow(topRow),
                  const SizedBox(height: 8),
                  _buildDiceRow(bottomRow),
                ],
              ),
            ),
            const SizedBox(width: 8),
            SizedBox(
              width: 96,
              child: FilledButton(
                style: FilledButton.styleFrom(
                  backgroundColor: const Color(0xFF414143),
                  foregroundColor: Colors.white,
                ),
                onPressed: !rollEnabled
                    ? null
                    : (_rolled ? _clear : _roll),
                child: Text(_rolled ? 'Clear' : 'Roll'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDiceRow(List<DiceType> row, {int totalSlots = 4}) {
    final children = <Widget>[];
    for (var i = 0; i < totalSlots; i++) {
      if (i > 0) children.add(const SizedBox(width: 8));
      if (i < row.length) {
        final t = row[i];
        children.add(
          Expanded(
            child: _DiceButton(
              type: t,
              enabled: !_rolled,
              onPressed: () => _enqueue(t),
            ),
          ),
        );
      } else {
        children.add(const Expanded(child: SizedBox()));
      }
    }
    return Row(children: children);
  }
}

class _DiceButton extends StatelessWidget {
  const _DiceButton({
    required this.type,
    required this.enabled,
    required this.onPressed,
  });

  final DiceType type;
  final bool enabled;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 64,
      child: OutlinedButton(
        onPressed: enabled ? onPressed : null,
        style: OutlinedButton.styleFrom(padding: EdgeInsets.zero),
        child: DiceIcon(type: type, size: type == DiceType.d4 ? 44 : 56),
      ),
    );
  }
}

class _QueueRow extends StatelessWidget {
  const _QueueRow({
    required this.type,
    required this.count,
    required this.subtotal,
  });

  final DiceType type;
  final int count;
  final int? subtotal;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          DiceIcon(type: type, size: 44),
          const SizedBox(width: 12),
          Text(
            'x$count',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const Spacer(),
          if (subtotal != null)
            Text(
              '= $subtotal',
              style: Theme.of(context).textTheme.titleMedium,
            ),
        ],
      ),
    );
  }
}
