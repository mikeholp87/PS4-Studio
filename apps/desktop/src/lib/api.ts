// Typed wrappers over the Tauri commands declared in src-tauri/build.rs. No other file calls invoke().
import { invoke } from '@tauri-apps/api/core'
export { listen } from '@tauri-apps/api/event'

export type Status = 'available' | 'unavailable' | 'unknown'
export type Tier = 'stock' | 'remote_play' | 'homebrew' | 'companion'
export interface CapabilityEntry {
  capability: string
  tier: Tier
  status: Status
  reason: string | null
  evidence: string | null
}
export type ConsoleModel = 'phat' | 'slim' | 'pro' | 'unknown'
export type FirmwareSource = 'unknown' | 'reported' | 'user_entered'
export interface ConsoleProfile {
  id: number
  name: string
  model: ConsoleModel
  address: string | null
  firmware: { value: string | null; source: FirmwareSource }
  simulated: boolean
  last_connected: number | null
}
export interface ConnectionView {
  profile: ConsoleProfile
  scenario_id: string | null
  simulated: boolean
  write_unlocked: boolean
  capabilities: CapabilityEntry[]
}
export interface Scenario {
  id: string
  label: string
  description: string
  console_name: string
  reported_firmware: string | null
  simulated: boolean
}
export interface AuditEntry { id: number; ts: number; action: string; target: string; detail: string; outcome: string }
export interface AppInfo { version: string; data_dir: string | null; config_dir: string | null }
export interface NewProfile { name: string; model: ConsoleModel; address: string; firmware: string | null }

export interface Discovered { address: string; name: string | null; state: 'ready' | 'standby'; firmware_hint: string | null }
export interface RemotePlayStatus { installed: boolean; binary: string | null; nickname: string | null; running: boolean }
export interface Check { id: string; title: string; level: 'pass' | 'warn' | 'fail' | 'info'; detail: string; hint: string | null }

export interface StoredTheme { id: string; name: string; json: string }

export interface Entry { name: string; kind: 'file' | 'dir' | 'link'; size: number }
export interface LocalListing { path: string; parent: string | null; entries: Entry[] }
export type RemoteOp =
  | { kind: 'upload'; local: string; remote: string }
  | { kind: 'mkdir'; path: string }
  | { kind: 'delete'; path: string; dir: boolean }
  | { kind: 'rename'; from: string; to: string }
export interface Plan { op: RemoteOp; changes: string[]; warnings: string[]; recovery: string; backup_remote: string | null; overwrites: boolean }
export interface PlanView { plan_id: string; plan: Plan }
export interface Outcome { lines: string[]; backup: string | null; verified: boolean; recovery: string }
export interface ApplyResult { outcome: Outcome; backup_id: string | null }
export interface BackupItem { category: 'settings' | 'profiles' | 'themes' | 'remote_file' | 'user_file'; original: string; stored: string; size: number; sha256: string }
export interface Manifest { version: number; id: string; created: number; label: string; items: BackupItem[] }
export interface VerifyReport { ok: boolean; problems: string[] }
export interface PackageInfo { file: string; size: number; manifest: { name: string; version: string; publisher: string; sha256: string } | null; trusted: boolean; warnings: string[] }
export interface HashResult { sha256: string; status: 'verified' | 'mismatch' | 'no_manifest' }

export const api = {
  appInfo: () => invoke<AppInfo>('app_info'),
  listScenarios: () => invoke<Scenario[]>('list_scenarios'),
  connectSimulator: (scenarioId: string) => invoke<ConnectionView>('connect_simulator', { scenarioId }),
  disconnect: () => invoke<void>('disconnect'),
  getConnection: () => invoke<ConnectionView | null>('get_connection'),
  listProfiles: () => invoke<ConsoleProfile[]>('list_profiles'),
  addProfile: (profile: NewProfile) => invoke<ConsoleProfile>('add_profile', { profile }),
  deleteProfile: (id: number) => invoke<boolean>('delete_profile', { id }),
  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),
  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),
  listAudit: () => invoke<AuditEntry[]>('list_audit'),
  discoverLan: () => invoke<Discovered[]>('discover_lan'),
  connectProfile: (id: number) => invoke<ConnectionView>('connect_profile', { id }),
  remotePlayStatus: () => invoke<RemotePlayStatus>('remote_play_status'),
  startRemotePlay: (nickname: string, fullscreen: boolean) => invoke<void>('start_remote_play', { nickname, fullscreen }),
  stopRemotePlay: () => invoke<void>('stop_remote_play'),
  runDiagnostics: () => invoke<{ checks: Check[] }>('run_diagnostics'),
  listThemes: () => invoke<StoredTheme[]>('list_themes'),
  saveTheme: (json: string) => invoke<StoredTheme>('save_theme', { json }),
  deleteTheme: (id: string) => invoke<boolean>('delete_theme', { id }),
  exportTheme: (id: string) => invoke<string | null>('export_theme', { id }),
  importTheme: () => invoke<StoredTheme | null>('import_theme'),
  filesConnect: (user: string | null, password: string | null) => invoke<ConnectionView>('files_connect', { user, password }),
  filesDisconnect: () => invoke<ConnectionView>('files_disconnect'),
  filesListRemote: (path: string) => invoke<Entry[]>('files_list_remote', { path }),
  filesListLocal: (path: string | null) => invoke<LocalListing>('files_list_local', { path }),
  filesSetWriteLock: (unlocked: boolean, acknowledgeUnencrypted: boolean) => invoke<ConnectionView>('files_set_write_lock', { unlocked, acknowledgeUnencrypted }),
  filesPlan: (op: RemoteOp) => invoke<PlanView>('files_plan', { op }),
  filesApply: (planId: string) => invoke<ApplyResult>('files_apply', { planId }),
  filesCancel: () => invoke<void>('files_cancel'),
  filesDownload: (remote: string, localDir: string) => invoke<string>('files_download', { remote, localDir }),
  backupCreate: (label: string) => invoke<Manifest>('backup_create', { label }),
  backupList: () => invoke<Manifest[]>('backup_list'),
  backupVerify: (id: string) => invoke<VerifyReport>('backup_verify', { id }),
  backupDelete: (id: string) => invoke<void>('backup_delete', { id }),
  backupRestoreItem: (id: string, index: number) => invoke<string>('backup_restore_item', { id, index }),
  backupSaveItemAs: (id: string, index: number) => invoke<string | null>('backup_save_item_as', { id, index }),
  hbPickAndScan: () => invoke<PackageInfo[] | null>('hb_pick_and_scan'),
  hbVerify: (name: string) => invoke<HashResult>('hb_verify', { name }),
  hbUploadOp: (name: string, acknowledgeUnverified = false) =>
    invoke<RemoteOp>('hb_upload_op', { name, acknowledgeUnverified }),
  companionConnect: (code: string) => invoke<ConnectionView>('companion_connect', { code }),
  companionSendTheme: (id: string) => invoke<void>('companion_send_theme', { id }),
  companionDisconnect: () => invoke<ConnectionView>('companion_disconnect'),
  exportDiagnostics: () => invoke<string>('export_diagnostics'),
}

export const errText = (e: unknown): string => (typeof e === 'string' ? e : e instanceof Error ? e.message : 'Unexpected error')
