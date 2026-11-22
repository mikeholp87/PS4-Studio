export type TemplateId = 'minimal' | 'aurora' | 'grid' | 'classic'
export type IconName = 'gamepad' | 'tv' | 'music' | 'film' | 'globe' | 'settings' | 'folder' | 'store' | 'users' | 'camera' | 'star' | 'download' | 'play' | 'image'
export const ICONS: IconName[] = ['gamepad', 'tv', 'music', 'film', 'globe', 'settings', 'folder', 'store', 'users', 'camera', 'star', 'download', 'play', 'image']
export const TEMPLATE_IDS: TemplateId[] = ['minimal', 'aurora', 'grid', 'classic']

export interface Tile { id: string; label: string; icon: IconName; color: string; kind: 'app' | 'shortcut' }

export interface Theme {
  schemaVersion: 1
  id: string
  name: string
  template: TemplateId
  background: { kind: 'solid' | 'gradient'; from: string; to: string; angle: number }
  colors: { accent: string; text: string; surface: string }
  font: { family: 'system' | 'serif' | 'mono' | 'rounded'; scale: number }
  layout: { columns: number; tileSize: 'small' | 'medium' | 'large'; tileShape: 'square' | 'rounded' | 'circle' }
  navigation: { wrapFocus: boolean; showLabels: boolean }
  widgets: { clock: { enabled: boolean; hour24: boolean; position: 'top-left' | 'top-right' }; status: { enabled: boolean } }
  tiles: Tile[]
}
