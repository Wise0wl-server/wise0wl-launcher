import React from 'react';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  List,
  Typography,
  IconButton,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Avatar,
  Chip,
  Button,
  Stack,
} from '@mui/material';
import {
  Menu as MenuIcon,
  PlayArrow as PlayIcon,
  AccountCircle as AccountIcon,
  Brightness4 as DarkModeIcon,
  Brightness7 as LightModeIcon,
  Article as ChangelogIcon,
  Login as LoginIcon,
  Logout as LogoutIcon,
  Settings as SettingsIcon,
} from '@mui/icons-material';
import { useTheme } from '../../contexts/ThemeContext';
import { useModpack } from '../../contexts/ModpackContext';
import { useUI } from '../../contexts/UIContext';

const drawerWidth = 240;

export const MainLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { darkMode, toggleDarkMode } = useTheme();
  const { 
    selectedModpack, 
    setSelectedModpack, 
    modpacks,
    user,
    isAuthenticated,
    login,
    logout
  } = useModpack();
  const { setSettingsOpen, setChangelogOpen } = useUI();

  // Sidebar content
  const drawer = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* User Info */}
      <Box sx={{ p: 2, display: 'flex', flexDirection: 'column', alignItems: 'center', borderBottom: 1, borderColor: 'divider' }}>
        <Avatar sx={{ width: 64, height: 64, mb: 1, bgcolor: 'primary.main', fontSize: '1.5rem' }}>
          {isAuthenticated && user ? user.name.charAt(0).toUpperCase() : <AccountIcon fontSize="large" />}
        </Avatar>
        <Typography variant="h6" component="div" sx={{ fontWeight: 'bold' }}>
          {isAuthenticated && user ? user.name : 'Not signed in'}
        </Typography>
        <Button
          variant="outlined"
          size="small"
          startIcon={isAuthenticated ? <LogoutIcon /> : <LoginIcon />}
          sx={{ mt: 1, mb: 1, textTransform: 'none' }}
          onClick={() => (isAuthenticated ? logout() : login())}
        >
          {isAuthenticated ? 'Logout' : 'Login'}
        </Button>
        <Chip label="Wise0wl Launcher" size="small" variant="outlined" />
      </Box>

      {/* Modpack List */}
      <Box sx={{ flex: 1, overflowY: 'auto', mt: 2 }}>
        <Typography variant="subtitle2" sx={{ pl: 2, mb: 1, color: 'text.secondary', fontWeight: 'bold' }}>
          Modpacks
        </Typography>
        <List>
          {modpacks.map((modpack) => (
            <ListItem key={modpack.id} disablePadding>
              <ListItemButton
                selected={selectedModpack?.id === modpack.id}
                onClick={() => setSelectedModpack(modpack.id)}
                sx={{ mx: 1, borderRadius: 1 }}
              >
                <ListItemIcon><PlayIcon /></ListItemIcon>
                <ListItemText primary={modpack.name} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Box>

      {/* Bottom Actions */}
      <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider' }}>
        <Stack direction="row" spacing={1} justifyContent="center">
          <Button
            startIcon={<SettingsIcon />}
            size="small"
            variant="text"
            onClick={() => setSettingsOpen(true)}
            sx={{ color: darkMode ? 'primary.contrastText' : 'inherit' }}
          >
            Settings
          </Button>
          <Button
            startIcon={<ChangelogIcon />}
            size="small"
            variant="text"
            onClick={() => setChangelogOpen(true)}
            sx={{ color: darkMode ? 'primary.contrastText' : 'inherit' }}
          >
            Changelog
          </Button>
        </Stack>
      </Box>
    </Box>
  );

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      {/* App Bar */}
      <AppBar
        position="fixed"
        sx={{
          width: { md: `calc(100% - ${drawerWidth}px)` },
          ml: { md: `${drawerWidth}px` },
          boxShadow: 1,
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            sx={{ mr: 2, display: { md: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            Wise0wl Launcher
          </Typography>
          <IconButton
            color="inherit"
            onClick={toggleDarkMode}
            aria-label="toggle dark mode"
          >
            {darkMode ? <LightModeIcon /> : <DarkModeIcon />}
          </IconButton>
        </Toolbar>
      </AppBar>

      {/* Permanent Sidebar */}
      <Drawer
        variant="permanent"
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          [`& .MuiDrawer-paper`]: { width: drawerWidth, boxSizing: 'border-box' },
        }}
        open
      >
        {drawer}
      </Drawer>

      {/* Main Content */}
      <Box component="main" sx={{ flexGrow: 1, p: 3, overflow: 'auto' }}>
        {children}
      </Box>
    </Box>
  );
}; 