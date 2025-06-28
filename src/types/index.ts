// Shared types between frontend and backend

export interface OnlineModpack {
  id: string;
  name: string;
  description: string;
  version: string;
  minecraft_version: string;
  modloader: string;
  modloader_version: string;
  image: string;
  url: string;
  scopes?: string[];
}

export interface ModFileEntry {
  filename: string;
  url: string;
  dir: string;
  hash?: string;
  hashformat?: string;
  scopes?: string[];
}

export interface Mod {
  id: string;
  name: string;
  version: string;
  required: boolean;
  downloadUrl: string;
  hash?: string;
}

export interface Modpack {
  id: string;
  name: string;
  description: string;
  version: string;
  minecraftVersion: string;
  forgeVersion?: string;
  fabricVersion?: string;
  image: string;
  mods: Mod[];
  lastUpdated: string;
  changelog?: string;
}

export interface Settings {
  javaPath: string;
  maxMemory: number;
  minMemory: number;
  gameResolution: {
    width: number;
    height: number;
  };
  gameDirectory: string;
}

export interface LaunchStatus {
  status: 'idle' | 'checking' | 'downloading' | 'launching' | 'running' | 'error';
  progress: number;
  message: string;
  error?: string;
}

export interface LaunchOptions {
  modpackId: string;
  gameDir: string;
  javaPath?: string;
  maxMemory?: number;
  minMemory?: number;
  width?: number;
  height?: number;
  accessToken: string;
  uuid: string;
  username: string;
}

export interface MinecraftVersionRequest {
  minecraftVersion: string;
}

export interface ACLUser {
  username: string;
  uuid: string;
  groups: string[];
}

export interface AuthToken {
  access_token: string;
  client_token: string;
  uuid: string;
  name: string;
  expires_at: number;
} 