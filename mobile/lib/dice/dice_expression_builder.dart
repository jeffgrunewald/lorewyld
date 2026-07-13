// Tap-to-compose builder for dice expression strings ("2d10 + 1d8 + 20").
// Mirrors the web implementation in server/assets/lw-content.js.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'dice_icon.dart';
import 'dice_type.dart';

// d100 intentionally omitted from the builder.
const List<DiceType> builderDice = [
  DiceType.d4,
  DiceType.d6,
  DiceType.d8,
  DiceType.d10,
  DiceType.d12,
  DiceType.d20,
];

const int _maxDieCount = 99;
const int _maxModifier = 9999;

final Set<int> _allowedSides = {for (final t in builderDice) t.sides};

// Terms render in insertion order; modifiers <= 0 drop; no terms -> "".
String formatDiceExpression(Map<int, int> terms, int modifier) {
  final parts = [
    for (final e in terms.entries)
      if (e.value > 0) '${e.value}d${e.key}',
    if (modifier > 0) '$modifier',
  ];
  return parts.join(' + ');
}

// Tolerates "18d10+72", "D8" (count 1); null when not a plain sum of
// allowed dice plus at most one flat bonus.
({Map<int, int> terms, int modifier})? parseDiceExpression(String input) {
  final terms = <int, int>{};
  final trimmed = input.trim();
  if (trimmed.isEmpty) return (terms: terms, modifier: 0);
  if (RegExp(r'[^0-9dD+\s]').hasMatch(trimmed)) return null;

  final dieRe = RegExp(r'^(\d*)\s*[dD]\s*(\d+)$');
  var modifier = 0;
  var sawModifier = false;
  for (final rawPart in trimmed.split('+')) {
    final part = rawPart.trim();
    if (part.isEmpty) return null;
    final die = dieRe.firstMatch(part);
    if (die != null) {
      final count = die.group(1)!.isEmpty ? 1 : int.parse(die.group(1)!);
      final sides = int.parse(die.group(2)!);
      if (count < 1 || !_allowedSides.contains(sides)) return null;
      final total = (terms[sides] ?? 0) + count;
      if (total > _maxDieCount) return null;
      terms[sides] = total;
    } else if (RegExp(r'^\d+$').hasMatch(part)) {
      final value = int.parse(part);
      if (sawModifier || value > _maxModifier) return null;
      modifier = value;
      sawModifier = true;
    } else {
      return null;
    }
  }
  return (terms: terms, modifier: modifier);
}

// Resolves to the composed expression on Select (possibly '', which is
// how a read-only field gets cleared), or null when dismissed.
Future<String?> showDiceExpressionBuilder(
  BuildContext context, {
  String initial = '',
}) {
  return showModalBottomSheet<String>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    builder: (_) => _DiceExpressionSheet(initial: initial),
  );
}

class _DiceExpressionSheet extends StatefulWidget {
  const _DiceExpressionSheet({required this.initial});

  final String initial;

  @override
  State<_DiceExpressionSheet> createState() => _DiceExpressionSheetState();
}

class _DiceExpressionSheetState extends State<_DiceExpressionSheet> {
  final Map<int, int> _terms = {};
  final _modifierCtl = TextEditingController();

  @override
  void initState() {
    super.initState();
    final parsed = parseDiceExpression(widget.initial);
    if (parsed != null) {
      _terms.addAll(parsed.terms);
      if (parsed.modifier > 0) _modifierCtl.text = '${parsed.modifier}';
    }
  }

  @override
  void dispose() {
    _modifierCtl.dispose();
    super.dispose();
  }

  String get _expression => formatDiceExpression(
    _terms,
    int.tryParse(_modifierCtl.text) ?? 0,
  );

  void _addDie(DiceType type) {
    setState(() {
      final count = _terms[type.sides] ?? 0;
      if (count < _maxDieCount) _terms[type.sides] = count + 1;
    });
  }

  void _clear() {
    setState(() {
      _terms.clear();
      _modifierCtl.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final expression = _expression;
    return Padding(
      padding: EdgeInsets.only(
        bottom: MediaQuery.of(context).viewInsets.bottom,
      ),
      child: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(24, 16, 24, 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text('Build dice expression', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            for (final type in builderDice) _dieRow(theme, type),
            const SizedBox(height: 12),
            TextField(
              controller: _modifierCtl,
              keyboardType: TextInputType.number,
              inputFormatters: [
                FilteringTextInputFormatter.digitsOnly,
                LengthLimitingTextInputFormatter(4),
              ],
              decoration: const InputDecoration(
                labelText: 'Modifier',
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 12),
            InputDecorator(
              decoration: const InputDecoration(
                labelText: 'Result',
                border: OutlineInputBorder(),
              ),
              child: Text(
                expression.isEmpty ? 'Tap dice to build…' : expression,
                style: expression.isEmpty
                    ? theme.textTheme.bodyLarge?.copyWith(
                        color: theme.colorScheme.outline,
                      )
                    : theme.textTheme.bodyLarge,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(onPressed: _clear, child: const Text('Clear')),
                const SizedBox(width: 8),
                FilledButton(
                  onPressed: () => Navigator.pop(context, expression),
                  child: const Text('Select'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _dieRow(ThemeData theme, DiceType type) {
    final count = _terms[type.sides] ?? 0;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 88,
            height: 56,
            child: OutlinedButton(
              onPressed: () => _addDie(type),
              style: OutlinedButton.styleFrom(padding: EdgeInsets.zero),
              child: DiceIcon(
                type: type,
                size: type == DiceType.d4 ? 40 : 48,
              ),
            ),
          ),
          const SizedBox(width: 16),
          Text(
            count > 0 ? '×$count' : '',
            style: theme.textTheme.titleMedium,
          ),
        ],
      ),
    );
  }
}
