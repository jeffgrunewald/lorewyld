import 'package:flutter/material.dart';

import '../home_screen.dart';
import 'lorewyld_app_bar.dart';

class LorewyldShell extends StatefulWidget {
  const LorewyldShell({super.key});

  @override
  State<LorewyldShell> createState() => _LorewyldShellState();
}

class _LorewyldShellState extends State<LorewyldShell> {
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();
  late final _ShellObserver _observer;
  bool _canPop = false;

  @override
  void initState() {
    super.initState();
    _observer = _ShellObserver(
      onChanged: (canPop) {
        if (mounted && canPop != _canPop) {
          setState(() => _canPop = canPop);
        }
      },
    );
  }

  Future<void> _popInnerRoute() async {
    await _navigatorKey.currentState?.maybePop();
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: !_canPop,
      onPopInvokedWithResult: (didPop, _) {
        if (didPop) return;
        _popInnerRoute();
      },
      child: Scaffold(
        appBar: LorewyldAppBar(
          leading: _canPop
              ? IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: _popInnerRoute,
                  tooltip: 'Back',
                )
              : null,
        ),
        body: Navigator(
          key: _navigatorKey,
          observers: [_observer],
          onGenerateRoute: (settings) => MaterialPageRoute(
            settings: settings,
            builder: (_) => const HomeScreen(),
          ),
        ),
      ),
    );
  }
}

class _ShellObserver extends NavigatorObserver {
  _ShellObserver({required this.onChanged});

  final ValueChanged<bool> onChanged;

  void _emit() {
    onChanged(navigator?.canPop() ?? false);
  }

  @override
  void didPush(Route<dynamic> route, Route<dynamic>? previousRoute) {
    super.didPush(route, previousRoute);
    WidgetsBinding.instance.addPostFrameCallback((_) => _emit());
  }

  @override
  void didPop(Route<dynamic> route, Route<dynamic>? previousRoute) {
    super.didPop(route, previousRoute);
    WidgetsBinding.instance.addPostFrameCallback((_) => _emit());
  }

  @override
  void didReplace({Route<dynamic>? newRoute, Route<dynamic>? oldRoute}) {
    super.didReplace(newRoute: newRoute, oldRoute: oldRoute);
    WidgetsBinding.instance.addPostFrameCallback((_) => _emit());
  }

  @override
  void didRemove(Route<dynamic> route, Route<dynamic>? previousRoute) {
    super.didRemove(route, previousRoute);
    WidgetsBinding.instance.addPostFrameCallback((_) => _emit());
  }
}
