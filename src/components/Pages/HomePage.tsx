import { Box, Typography, Button, LinearProgress, Alert } from '@mui/material';
import { useModpack } from '../../contexts/ModpackContext';

export const HomePage = () => {
  const { 
    selectedModpack, 
    launchStatus, 
    launchGame,
    isAuthenticated,
    login
  } = useModpack();

  const handlePlay = async () => {
    try {
      await launchGame();
    } catch (error) {
      console.error('Failed to launch game:', error);
    }
  };

  return (
    <Box
      sx={{
        position: 'relative',
        width: '100%',
        height: 'calc(100vh - 48px)', // leave space for app bar
        minHeight: 400,
        overflow: 'hidden',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        p: 0,
        m: 0,
        mx: 0,
      }}
    >
      {/* Full-bleed Modpack Image */}
      {selectedModpack && (
        <Box
          component="img"
          src={selectedModpack.image}
          alt={selectedModpack.name}
          sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            objectFit: 'cover',
            filter: 'brightness(0.7)',
            zIndex: 1,
          }}
        />
      )}

      {/* Overlay: Name & Description */}
      {selectedModpack && (
        <Box sx={{ position: 'absolute', top: 0, left: 0, width: '100%', p: 5, color: 'primary.contrastText', zIndex: 2 }}>
          <Typography variant="h3" sx={{ fontWeight: 'bold', textShadow: '0 2px 8px #000' }}>
            {selectedModpack.name}
          </Typography>
          <Typography variant="h5" sx={{ mt: 2, textShadow: '0 2px 8px #000' }}>
            {selectedModpack.description}
          </Typography>
          <Typography variant="body1" sx={{ mt: 2, textShadow: '0 2px 8px #000' }}>
            Minecraft {selectedModpack.minecraftVersion}
            {selectedModpack.forgeVersion && ` • Forge ${selectedModpack.forgeVersion}`}
            {selectedModpack.fabricVersion && ` • Fabric ${selectedModpack.fabricVersion}`}
          </Typography>
        </Box>
      )}

      {/* Error Alert */}
      {launchStatus.status === 'error' && (
        <Box sx={{ position: 'absolute', top: 16, right: 16, zIndex: 3 }}>
          <Alert severity="error" variant="filled">
            {launchStatus.error || 'Failed to launch game'}
          </Alert>
        </Box>
      )}

      {/* Overlay: Play Button or Login Button */}
      <Box sx={{ position: 'absolute', bottom: 64, left: 0, width: '100%', display: 'flex', justifyContent: 'center', zIndex: 3 }}>
        {isAuthenticated ? (
          <Button
            variant="contained"
            size="large"
            sx={{ px: 6, py: 1.5, fontSize: 28, borderRadius: 3, boxShadow: '0 4px 16px rgba(0,0,0,0.3)' }}
            onClick={handlePlay}
            disabled={launchStatus.status !== 'idle' && launchStatus.status !== 'error'}
          >
            {launchStatus.status === 'idle' || launchStatus.status === 'error' ? 'Play' : launchStatus.message}
          </Button>
        ) : (
          <Button
            variant="contained"
            size="large"
            sx={{ px: 6, py: 1.5, fontSize: 28, borderRadius: 3, boxShadow: '0 4px 16px rgba(0,0,0,0.3)' }}
            onClick={login}
          >
            Login with Microsoft
          </Button>
        )}
      </Box>

      {/* Overlay: Progress Bar */}
      {launchStatus.status !== 'idle' && launchStatus.status !== 'error' && (
        <Box sx={{ position: 'absolute', bottom: 0, left: 0, width: '100%', zIndex: 4 }}>
          <LinearProgress 
            variant="determinate" 
            value={launchStatus.progress} 
            sx={{ height: 10, borderRadius: 0 }} 
          />
          <Typography 
            variant="body2" 
            color="primary.contrastText" 
            sx={{ position: 'absolute', left: 16, bottom: 12, textShadow: '0 2px 8px #000' }}
          >
            {launchStatus.message}
          </Typography>
        </Box>
      )}
    </Box>
  );
}; 