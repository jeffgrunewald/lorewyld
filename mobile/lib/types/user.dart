// Mirror of `lorewyld_types::user::User`. Hand-written: typeshare-cli
// no longer ships a Dart backend, so these types track the Rust crate
// manually.

class User {
  final String uuid;
  final String username;
  final String email;
  final bool admin;
  final DateTime createdAt;

  const User({
    required this.uuid,
    required this.username,
    required this.email,
    required this.admin,
    required this.createdAt,
  });

  factory User.fromJson(Map<String, dynamic> json) => User(
    uuid: json['uuid'] as String,
    username: json['username'] as String,
    email: json['email'] as String,
    admin: json['admin'] as bool,
    createdAt: DateTime.parse(json['created_at'] as String),
  );

  Map<String, dynamic> toJson() => {
    'uuid': uuid,
    'username': username,
    'email': email,
    'admin': admin,
    'created_at': createdAt.toIso8601String(),
  };
}
