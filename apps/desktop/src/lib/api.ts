// Typed wrappers over the Tauri commands declared in src-tauri/build.rs. No other file calls invoke().
import { invoke } from '@tauri-apps/api/core'

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
  exportDiagnostics: () => invoke<string>('export_diagnostics'),
}

export const errText = (e: unknown): string => (typeof e === 'string' ? e : e instanceof Error ? e.message : 'Unexpected error')
