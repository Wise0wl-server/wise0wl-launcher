import React, { useState } from 'react';
import { Button, Typography, Box, Alert, Chip, CircularProgress } from '@mui/material';
import { useModpack } from '../../contexts/ModpackContext';

export const MicrosoftAuth: React.FC = () => {
  const { login, logout, user, isAuthenticated, isAuthLoading } = useModpack();
  const [error, setError] = useState<string | null>(null);

  const handleMicrosoftLogin = async () => {
    setError(null);
    try {
      await login();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start Microsoft authentication');
    }
  };

  if (isAuthenticated && user) {
    return (
      <Box sx={{ p: 2 }}>
        <Typography variant="h6" gutterBottom>
          Microsoft Authentication
        </Typography>
        
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
          <Chip 
            label="Authenticated" 
            color="success" 
            variant="outlined"
          />
          <Typography variant="body1">
            Signed in as: <strong>{user.name}</strong>
          </Typography>
        </Box>

        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          You are now authenticated and can launch Minecraft.
        </Typography>

        <Button
          variant="outlined"
          onClick={logout}
          disabled={isAuthLoading}
          sx={{ mb: 2 }}
        >
          Sign Out
        </Button>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        Microsoft Authentication
      </Typography>
      
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
        Sign in with your Microsoft account to access Minecraft Java Edition.
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {isAuthLoading && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
          <CircularProgress size={20} />
          <Typography variant="body2" color="text.secondary">
            Processing authentication...
          </Typography>
        </Box>
      )}

      <Button
        variant="contained"
        onClick={handleMicrosoftLogin}
        disabled={isAuthLoading}
        sx={{ mb: 2 }}
      >
        {isAuthLoading ? 'Processing...' : 'Sign in with Microsoft'}
      </Button>

      <Typography variant="caption" display="block" color="text.secondary">
        You will be redirected to the Microsoft login page.
      </Typography>
    </Box>
  );
}; 