# Companion restore points before theme deploy

**Date:** 2026-11-26  
**Status:** Approved design (Approach A)  
**Owner area:** companion protocol, `crates/backups`, Tauri `companion_*` / `backup_*`, Studio send dialog, Backups UI

## Problem

Sending a theme to the PS4 Studio companion overwrites companion-stored themes with no way to put the previous set back. Users asked for a backup before design deploy so they can revert. A full PlayStation OS / official home-screen backup is **not** possible with this app (stock PS4 has no such API; system trees are read-only; security model forbids automatic console rollback promises).

## Goal

Before deploying a Dashboard Studio theme to the companion, create a local checksummed **restore point** of everything the companion currently holds that we can put back: all theme JSON payloads and `active_theme`. Provide one-click restore. Copy must state clearly that this reverts **companion themes only**, not the official PS4 home screen or system software.

## Non-goals

- Full OS / system-partition / firmware images
- Automating Sony’s official USB/online backup
- FTP `/data` sweeps (themes live in the companion today)
- Encrypting the companion link (still authenticated, not encrypted)
- Restoring desktop Dashboard Studio library themes (already local files)

## Architecture

```
Studio "Send" → [optional snapshot] → put_theme
                      │
                      ▼
         list_themes + get_theme* + get_config(active_theme)
                      │
                      ▼
              backups::create (local, SHA-256 items)
                      │
Backups UI / dialog ←── restore → put_theme* + set_config + delete extras
```

Units:

| Unit | Responsibility | Depends on |
|---|---|---|
| Protocol `get_theme` | Return one theme’s validated JSON by id | Existing frame/MAC/allowlist |
| Snapshot builder | Collect companion state into `NewItem`s | Companion client, `backups` |
| Send command | Gate: snapshot (default) then `put_theme` | Snapshot builder, theme store |
| Restore command | Apply snapshot to live companion | Companion client, `backups` |
| UI | Opt-out checkbox, honesty copy, restore entry points | Existing dialogs / Backups page |

## Protocol change

Keep **protocol version 1**. Add one allowlisted command:

- **Name:** `get_theme`
- **Request:** `{ "id": "<slug>" }` (same id rules as `put_theme`)
- **Success data:** `{ "id": "<slug>", "json": "<theme json text>" }`
- **Errors:** unknown id; invalid id; theme fails the same validation used on `put_theme`
- **Size:** subject to existing `MAX_FRAME` (400 KB); refuse oversized payloads before allocation (already framed)

Update in lockstep:

1. `crates/core/src/companion_proto.rs` — `COMMANDS`
2. `apps/ps4-companion/src/proto.c` — allowlist (+ host test)
3. `simulator/mock-console` companion handler
4. `crates/console-adapters` — `Client::get_theme` / `list_themes` helpers as needed
5. OpenOrbis skeleton comment/README command list (no requirement to implement console fetch beyond host/simulator for this phase)

## Backup data model

Reuse `crates/backups` directory manifests.

- Add category **`CompanionTheme`** for theme JSON bytes (`original` = theme id).
- Store active theme as a **`CompanionTheme`** item with `original` = `config:active_theme` and UTF-8 body = the config value string (empty string if unset).  
  Rationale: avoids a second category enum churn while remaining filterable by `original` prefix / category.

Manifest **label** pattern:

`Companion snapshot before sending "<theme-id>"`

Optional metadata is not added to the manifest schema in this phase (label + audit row are enough).

## Send flow

1. Preconditions (unchanged + extended):
   - Session has paired companion
   - `CompanionThemeManagement` is Available
   - Theme id exists in local `ThemeStore` and validates
2. UI default: **Create restore point** checked.
3. If checked:
   - `list_themes`
   - For each id: `get_theme` → collect bytes
   - `get_config("active_theme")`
   - `backups::create(...)` under XDG data `backups/`
   - Audit `companion_snapshot` with backup id / theme count / outcome
   - On any snapshot failure → **abort send**, return clear error (do not partially put_theme)
4. `put_theme(id, json)` as today
5. Audit `companion_send_theme` including `backup_id` when a snapshot was taken

If the user unchecks restore point: show a one-line warning in the dialog; allow send without snapshot.

Empty companion (no themes): still create a snapshot with only `config:active_theme` (possibly empty) so restore can clear later themes if desired. Alternatively: create a snapshot with zero theme items + config — restore deletes all companion themes then applies config. **Decision:** empty theme list is a valid snapshot; restore will delete any themes not in the snapshot (see below), so an empty pre-send snapshot restores to “no themes.”

## Restore flow

Preconditions: paired companion with theme management Available; backup exists and verifies.

1. `backup_verify(id)` must pass
2. Confirm dialog: “This replaces companion themes with the snapshot. It does not change the official PS4 home screen.”
3. Apply order:
   - `list_themes` current
   - `put_theme` for every theme item in the snapshot
   - `set_config("active_theme", value)` from `config:active_theme` item (if missing in old backups, skip config)
   - `delete_theme` for each current id **not** present in the snapshot theme set
4. Audit `companion_restore` with backup id and counts
5. On mid-restore failure: stop, report which step failed, leave companion in partial state with honest message (no false “rolled back” claim). User can retry restore.

Entry points:

- **Backups** page: restore action enabled for manifests that contain at least one `CompanionTheme` item
- **Companion send dialog**: “Restore last companion snapshot…” when the newest companion-tagged backup exists (optional convenience; Backups remains canonical)

## UI copy (required)

Use consistently:

> Restores PS4 Studio companion themes only. It does not change the official PS4 home screen or system software.

Send dialog checkbox label:

> Create a restore point first (recommended)

## Error handling

| Case | Behaviour |
|---|---|
| `get_theme` unsupported / refused | Snapshot fails; send aborted if restore point required |
| Theme missing mid-list (deleted concurrently) | Snapshot fails closed; abort send |
| Backup disk full / create error | Abort send |
| Restore with companion disconnected | Refuse before mutating |
| Verify fails | Refuse restore |

## Testing

1. **Simulator unit/integration:** seed two themes + active → snapshot → put third → restore → list and active match seed; extras removed.
2. **Snapshot failure:** mock `get_theme` error → send with restore point on fails; companion unchanged.
3. **Empty companion:** snapshot then send → restore removes the sent theme.
4. **C host test:** `get_theme` in allowlist; unknown command still refused.
5. **IPC (as practical):** extend real-ACL flow: pair → seed via put → snapshot path → send → restore.
6. **UI:** checkbox default on; honesty string visible on send and restore.

## Documentation

- `docs/security.md` — companion restore points; still no OS rollback promise
- `docs/tasks.md` / `current-status.md` — handoff entry when implemented
- README companion bullet — one honesty line
- Project site companion card — optional one-liner in a later docs pass (not blocking)

## Rollout / honesty

Ship as companion feature, still **simulator-verified** until hardware companion exists. Do not market as “full PS4 backup.”

## Implementation order (for the plan)

1. Protocol + mock + C allowlist + adapter `get_theme`
2. Backup category + snapshot/restore Rust API
3. Tauri commands + audit
4. Companion dialog + Backups UI
5. Docs + gates
