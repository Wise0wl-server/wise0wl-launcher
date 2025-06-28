import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-shell';
import {
  OnlineModpack,
  Modpack,
  Settings,
  LaunchStatus,
  LaunchOptions,
  ACLUser,
  AuthToken
} from '../types';

interface ModpackContextType {
  modpacks: Modpack[];
  selectedModpack: Modpack | null;
  setSelectedModpack: (id: string | null) => void;
  settings: Settings | null;
  saveSettings: (settings: Settings) => Promise<void>;
  launchGame: () => Promise<void>;
  launchStatus: LaunchStatus;
  onlineModpacks: OnlineModpack[];
  fetchOnlineModpacks: (listUrl: string) => Promise<void>;
  installOnlineModpack: (modpack: OnlineModpack) => Promise<void>;
  userGroups: string[];
  setUserUuid: (uuid: string) => void;
  getMicrosoftAuthUrl: () => Promise<{ url: string; state: string }>;
  handleMicrosoftCallback: (code: string, state: string) => Promise<AuthToken>;
  authToken: AuthToken | null;
  user: { uuid: string; name: string } | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  isAuthenticated: boolean;
  isAuthLoading: boolean;
}

const ModpackContext = createContext<ModpackContextType | undefined>(undefined);

export const useModpack = () => {
  const context = useContext(ModpackContext);
  if (!context) throw new Error('useModpack must be used within a ModpackProvider');
  return context;
};

// Example: Replace with your actual ACL source (local or remote)
const ACL_URL = '/acl.json';

export const ModpackProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [modpacks, setModpacks] = useState<Modpack[]>([]);
  const [selectedModpackId, setSelectedModpackId] = useState<string | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [launchStatus, setLaunchStatus] = useState<LaunchStatus>({
    status: 'idle',
    progress: 0,
    message: 'Ready to play',
  });
  const [onlineModpacks, setOnlineModpacks] = useState<OnlineModpack[]>([]);
  const [userGroups, setUserGroups] = useState<string[]>(['default']);
  const [authToken, setAuthToken] = useState<AuthToken | null>(null);
  const [isAuthLoading, setIsAuthLoading] = useState(false);
  const [oauthState, setOauthState] = useState<string | null>(null);

  const selectedModpack = modpacks.find(p => p.id === selectedModpackId) ?? null;

  // Check for stored user session on startup
  useEffect(() => {
    const checkStoredSession = async () => {
      const storedUuid = localStorage.getItem('userUuid');
      if (storedUuid) {
        try {
          console.log('Checking for stored session with UUID:', storedUuid);
          const token = await invoke<AuthToken | null>('get_auth_token', { uuid: storedUuid });
          if (token) {
            console.log('Found valid stored session for user:', token.name);
            setAuthToken(token);
            // Fetch user groups after successful reconnection
            handleSetUserUuid(token.uuid);
          } else {
            console.log('Stored session expired or invalid, clearing...');
            localStorage.removeItem('userUuid');
          }
        } catch (error) {
          console.error('Failed to retrieve stored auth token:', error);
          localStorage.removeItem('userUuid');
        }
      } else {
        console.log('No stored session found');
      }
    };
    checkStoredSession();
  }, []);

  // Listen for the custom protocol callback from the deep-link plugin
  useEffect(() => {
    const unlistenPromise = listen<string[]>('oauth_callback', (event) => {
      // The payload is an array of URLs, we take the first one
      const requestUrl = event.payload[0];
      console.log('Received OAuth callback:', event.payload);
      
      if (!requestUrl) {
        console.warn('No URL received in OAuth callback');
        return;
      }

      try {
        // Ensure the URL has a proper scheme
        let urlToParse = requestUrl;
        if (!urlToParse.startsWith('http://') && !urlToParse.startsWith('https://') && !urlToParse.startsWith('wise0wl-oauth://')) {
          // If it's just a path or query string, prepend the scheme
          if (urlToParse.startsWith('/')) {
            urlToParse = `wise0wl-oauth://callback${urlToParse}`;
          } else if (urlToParse.includes('code=')) {
            urlToParse = `wise0wl-oauth://callback?${urlToParse}`;
          } else {
            console.error('Invalid callback URL format:', requestUrl);
            return;
          }
        }

        console.log('Parsing URL:', urlToParse);
        const url = new URL(urlToParse);
        const code = url.searchParams.get('code');
        const state = url.searchParams.get('state') || localStorage.getItem('oauthState') || oauthState;
        if (code && state) {
          console.log('Found authorization code and state, processing callback...');
          handleMicrosoftCallback(code, state).catch(error => {
            console.error('Failed to handle Microsoft callback:', error);
            // You might want to show an error message to the user here
          });
          localStorage.removeItem('oauthState');
          setOauthState(null);
        } else {
          console.warn('No authorization code or state found in callback URL');
        }
      } catch (error) {
        console.error('Failed to parse callback URL:', error, 'URL was:', requestUrl);
      }
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [oauthState]);

  // Fetch the user's groups from the ACL using their UUID
  const fetchUserGroups = async (uuid: string) => {
    try {
      const resp = await fetch(ACL_URL);
      const acl: ACLUser[] = await resp.json();
      const user = acl.find(u => u.uuid === uuid);
      if (user) {
        setUserGroups(user.groups);
      } else {
        setUserGroups(['default']);
      }
    } catch (error) {
      setUserGroups(['default']);
      console.error('Failed to fetch user groups from ACL:', error);
    }
  };

  // Call this when the user logs in or their UUID is known
  const handleSetUserUuid = (uuid: string) => {
    fetchUserGroups(uuid);
  };

  useEffect(() => {
    const loadData = async () => {
      try {
        const [packs, loadedSettings] = await Promise.all([
          invoke<Modpack[]>('get_modpacks'),
          invoke<Settings>('get_settings'),
        ]);
        setModpacks(packs);
        setSettings(loadedSettings);
        if (packs.length > 0 && !selectedModpackId) {
          setSelectedModpackId(packs[0].id);
        }
      } catch (error) {
        console.error('Failed to load initial data:', error);
      }
    };
    loadData();
  }, [selectedModpackId]);

  const saveSettings = async (newSettings: Settings) => {
    try {
      await invoke('save_settings', { settings: newSettings });
      setSettings(newSettings);
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  };

  // Group-aware: Fetch online modpacks for the user's groups
  const fetchOnlineModpacks = async (listUrl: string) => {
    try {
      const packs = await invoke<OnlineModpack[]>('fetch_modpack_list', {
        listUrl,
        userGroups,
      });
      setOnlineModpacks(packs);
    } catch (error) {
      console.error('Failed to fetch online modpacks:', error);
    }
  };

  // Group-aware: Install an online modpack for the user's groups
  const installOnlineModpack = async (modpack: OnlineModpack) => {
    setLaunchStatus({
      status: 'downloading',
      progress: 0,
      message: `Downloading and installing ${modpack.name}...`,
    });
    try {
      await invoke('download_modpack_with_groups', {
        modpack,
        userGroups,
      });
      setLaunchStatus({
        status: 'launching',
        progress: 100,
        message: 'Modpack installed. Ready to launch!',
      });
    } catch (error) {
      setLaunchStatus({
        status: 'error',
        progress: 0,
        message: 'Failed to install modpack',
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      throw error;
    }
  };

  const getMicrosoftAuthUrl = async (): Promise<{ url: string; state: string }> => {
    try {
      return await invoke<{ url: string; state: string }>('get_microsoft_auth_url');
    } catch (error) {
      console.error('Failed to get Microsoft auth URL:', error);
      throw error;
    }
  };

  const handleMicrosoftCallback = async (code: string, state: string): Promise<AuthToken> => {
    setIsAuthLoading(true);
    try {
      console.log('Processing Microsoft OAuth callback...');
      const token = await invoke<AuthToken>('handle_microsoft_callback', { code, state });
      console.log('Successfully authenticated user:', token.name);
      setAuthToken(token);
      localStorage.setItem('userUuid', token.uuid);
      // Fetch user groups after successful authentication
      handleSetUserUuid(token.uuid);
      return token;
    } catch (error) {
      console.error('Failed to handle Microsoft callback:', error);
      throw error;
    } finally {
      setIsAuthLoading(false);
    }
  };

  const login = async () => {
    setIsAuthLoading(true);
    try {
      console.log('Starting Microsoft OAuth flow...');
      const { url, state } = await getMicrosoftAuthUrl();
      setOauthState(state);
      localStorage.setItem('oauthState', state);
      console.log('Opening Microsoft login URL...');
      await open(url);
    } catch (error) {
      console.error('Failed to start Microsoft OAuth:', error);
      setIsAuthLoading(false);
      throw error;
    }
  };

  const logout = async () => {
    console.log('Logging out user...');
    if (authToken) {
      try {
        await invoke('logout_user', { uuid: authToken.uuid });
      } catch (error) {
        console.error('Failed to logout from backend:', error);
      }
    }
    setAuthToken(null);
    localStorage.removeItem('userUuid');
    setUserGroups(['default']);
  };

  const launchGame = async () => {
    if (!settings || !selectedModpack) {
      console.error('Settings or modpack not selected');
      setLaunchStatus({ status: 'error', progress: 0, message: 'Settings or modpack not loaded' });
      return;
    }

    if (!authToken) {
      console.error('Not authenticated');
      setLaunchStatus({ status: 'error', progress: 0, message: 'Please sign in first' });
      return;
    }

    setLaunchStatus({ status: 'checking', progress: 0, message: 'Preparing to launch...' });

    try {
      const javaPath = await invoke<string>('ensure_java_installed_for_mc', {
        request: { minecraftVersion: selectedModpack.minecraftVersion },
      });

      setLaunchStatus({ status: 'launching', progress: 50, message: 'Launching Minecraft...' });

      const launchOptions: LaunchOptions = {
        modpackId: selectedModpack.id,
        gameDir: settings.gameDirectory,
        javaPath,
        maxMemory: settings.maxMemory,
        minMemory: settings.minMemory,
        width: settings.gameResolution.width,
        height: settings.gameResolution.height,
        accessToken: authToken.access_token,
        uuid: authToken.uuid,
        username: authToken.name,
      };

      await invoke('launch_minecraft', { options: launchOptions });

      setLaunchStatus({ status: 'running', progress: 100, message: 'Minecraft is running!' });
    } catch (error) {
      setLaunchStatus({
        status: 'error',
        progress: 0,
        message: 'Failed to launch',
        error: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const contextValue: ModpackContextType = {
    modpacks,
    selectedModpack,
    setSelectedModpack: setSelectedModpackId,
    settings,
    saveSettings,
    launchGame,
    launchStatus,
    onlineModpacks,
    fetchOnlineModpacks,
    installOnlineModpack,
    userGroups,
    setUserUuid: handleSetUserUuid,
    getMicrosoftAuthUrl,
    handleMicrosoftCallback,
    authToken,
    user: authToken ? { uuid: authToken.uuid, name: authToken.name } : null,
    login,
    logout,
    isAuthenticated: !!authToken,
    isAuthLoading,
  };

  return <ModpackContext.Provider value={contextValue}>{children}</ModpackContext.Provider>;
}; 