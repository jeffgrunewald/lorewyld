import 'dart:math';

enum DiceType {
  d4(sides: 4, label: 'd4'),
  d6(sides: 6, label: 'd6'),
  d8(sides: 8, label: 'd8'),
  d10(sides: 10, label: 'd10'),
  d10Percentile(sides: 100, label: 'd100'),
  d12(sides: 12, label: 'd12'),
  d20(sides: 20, label: 'd20');

  const DiceType({required this.sides, required this.label});

  final int sides;
  final String label;

  static const List<DiceType> displayOrder = values;

  int roll(Random random) => random.nextInt(sides) + 1;
}
