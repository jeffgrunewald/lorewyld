// Mirror of `lorewyld_types::app_user::AppUser`. Hand-written for v1;
// typeshare-cli no longer ships a Dart backend, so until that's restored
// (or replaced) these types track the Rust crate manually.

class AppUser {
  final String uuid;
  final String serverUuid;
  final String displayName;
  final DateTime createdAt;

  const AppUser({
    required this.uuid,
    required this.serverUuid,
    required this.displayName,
    required this.createdAt,
  });

  factory AppUser.fromJson(Map<String, dynamic> json) => AppUser(
        uuid: json['uuid'] as String,
        serverUuid: json['server_uuid'] as String,
        displayName: json['display_name'] as String,
        createdAt: DateTime.parse(json['created_at'] as String),
      );

  Map<String, dynamic> toJson() => {
        'uuid': uuid,
        'server_uuid': serverUuid,
        'display_name': displayName,
        'created_at': createdAt.toIso8601String(),
      };
}
