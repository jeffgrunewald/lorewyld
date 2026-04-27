import 'package:flutter/material.dart';

import 'widgets/lorewyld_shell.dart';

void main() {
  runApp(const LorewyldApp());
}

class LorewyldApp extends StatelessWidget {
  const LorewyldApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Lorewyld',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const LorewyldShell(),
    );
  }
}
