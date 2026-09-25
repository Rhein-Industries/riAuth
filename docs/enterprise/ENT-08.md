# ENT-08 Password history

[Implementation](../../src/password_history.rs) and [tests](../../tests/password_history.rs).

riAuth can reject reuse of recent passwords. This is an operator control, not a certification of any compliance regime.

## Configuration

`password_history` in `riauth.toml` is the number of password hashes retained per user. The default is 5 when the key is omitted. `0` disables reuse checks. The maximum is 24; any larger value fails configuration validation and the process will not start.

```toml
password_history = 5
```

The running process reads the value on each password write. Restart after changing `riauth.toml`. The storage schema version stays at 3; existing databases grow a `password_history` record on the next accepted password write and do not require a migration.

## What is stored

History is a per-user list of password hashes only, newest last, in the `password_history` bucket keyed by user id. Plaintext passwords are not stored. The list is capped at `password_history` hashes, including the current password when it has been recorded. A limit of 5 normally means the current password plus four earlier recorded hashes, not five earlier passwords plus the current one. The account's current password hash is always checked, including when it is not yet in that list (accounts created before history was enabled, or a hash that has not been recorded).

A zero limit skips checks and does not append a new hash. Hashes already stored are left in the database and are ignored until a non-zero limit is configured again. Lowering a non-zero limit checks the current hash plus the newest stored hashes that fit the new limit. The next accepted change rewrites the row down to that cap, after which the dropped hash may be reused.

## Where it is enforced

The comparison runs in the same storage transaction as the user update. These writes participate:

- self-service password change
- email password reset and invitation acceptance
- administrator user update and local `recover-admin`
- user creation, including the bootstrap administrator (the initial hash is the first history entry)
- SCIM password set
- manifest apply of `password_ref` (plaintext) or `password_hash_ref` (imported hash)

Reuse returns `invalid_request` with `Password was used recently`. The response does not include the password or any hash.

## Imports, migration, and rehash

`password_hash_ref`, including hashes produced by the Authentik migration manifest, is recorded with `record_imported_hash`. No plaintext is available, so the import itself is not compared against history. The hash is retained. A later change, reset, SCIM write, or manifest `password_ref` that presents the same plaintext is rejected.

Publishing the identical hash again with a new `password_version` does not raise a reuse error; it records that hash. Two different hashes of one password each occupy a slot. A plaintext change is rejected if either hash is still in the checked window.

Login rehash (`pbkdf2_sha256` and `$argon2i$` upgraded to `$argon2id$`) is the same password. Login is not rejected. The upgraded hash replaces the previous hash in history when that previous hash is present; otherwise the upgraded hash is appended. It is not treated as reuse.

Emptying a password does not append history and does not remove hashes already retained. That includes `password_disabled` and a SCIM user created without a password. The next password is still checked against retained hashes. An empty hash is not a history entry.

Setting the same `password_version` again is not a password change and does not touch history. Changing `password_version` while supplying the current plaintext is a new password write and is rejected when history is enabled.

## What is not exposed

User views, SCIM resources, audit events, and plan output do not include password history or password hashes. Audit metadata for these writes records the actor, action, and public before/after resource view only.

Backups snapshot every record, including `password_history` and the current password hash. Treat a backup as credential material. Database encryption covers history the same way it covers other records.

Preview plans do not read or write history and do not require the referenced password secret to be loaded.
