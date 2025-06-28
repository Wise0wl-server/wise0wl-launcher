import React, { useState } from 'react';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';
import Grid from '@mui/material/Grid';
import FolderOpenIcon from '@mui/icons-material/FolderOpen';
import { useModpack } from '../../contexts/ModpackContext';
import { useUI } from '../../contexts/UIContext';

export const SettingsDialog: React.FC = () => {
  const { settings, saveSettings } = useModpack();
  const { settingsOpen, setSettingsOpen } = useUI();
  const [form, setForm] = useState({
    javaPath: '',
    maxMemory: 4096,
    minMemory: 1024,
    gameResolution: { width: 1280, height: 720 },
    gameDirectory: ''
  });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  React.useEffect(() => {
    if (settings) {
      setForm({
        javaPath: settings.javaPath ?? '',
        maxMemory: settings.maxMemory ?? 4096,
        minMemory: settings.minMemory ?? 1024,
        gameResolution: settings.gameResolution ?? { width: 1280, height: 720 },
        gameDirectory: settings.gameDirectory ?? ''
      });
    }
  }, [settings]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    if (name === 'width' || name === 'height') {
      setForm(f => ({ ...f, gameResolution: { ...f.gameResolution, [name]: Number(value) } }));
    } else if (name === 'maxMemory' || name === 'minMemory') {
      setForm(f => ({ ...f, [name]: Number(value) }));
    } else {
      setForm(f => ({ ...f, [name]: value }));
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await saveSettings(form);
      setSettingsOpen(false);
    } catch (e: any) {
      setError(e.message || 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={settingsOpen} onClose={() => setSettingsOpen(false)} maxWidth="sm" fullWidth>
      <DialogTitle>Launcher Settings</DialogTitle>
      <DialogContent>
        <Grid container spacing={2} sx={{ mt: 1 }}>
          <Grid size={12}>
            <TextField
              label="Java Path (optional)"
              name="javaPath"
              value={form.javaPath}
              onChange={handleChange}
              fullWidth
              helperText="Leave blank to use auto-managed Java runtime."
            />
          </Grid>
          <Grid size={6}>
            <TextField
              label="Max Memory (MB)"
              name="maxMemory"
              type="number"
              value={form.maxMemory}
              onChange={handleChange}
              fullWidth
              slotProps={{ input: { endAdornment: <InputAdornment position="end">MB</InputAdornment> } }}
            />
          </Grid>
          <Grid size={6}>
            <TextField
              label="Min Memory (MB)"
              name="minMemory"
              type="number"
              value={form.minMemory}
              onChange={handleChange}
              fullWidth
              slotProps={{ input: { endAdornment: <InputAdornment position="end">MB</InputAdornment> } }}
            />
          </Grid>
          <Grid size={6}>
            <TextField
              label="Resolution Width"
              name="width"
              type="number"
              value={form.gameResolution.width}
              onChange={handleChange}
              fullWidth
            />
          </Grid>
          <Grid size={6}>
            <TextField
              label="Resolution Height"
              name="height"
              type="number"
              value={form.gameResolution.height}
              onChange={handleChange}
              fullWidth
            />
          </Grid>
          <Grid size={12}>
            <TextField
              label="Game Directory"
              name="gameDirectory"
              value={form.gameDirectory}
              onChange={handleChange}
              fullWidth
              slotProps={{ input: {
                endAdornment: (
                  <InputAdornment position="end">
                    <FolderOpenIcon fontSize="small" />
                  </InputAdornment>
                ),
              }}}
            />
          </Grid>
        </Grid>
        {error && <div style={{ color: 'red', marginTop: 8 }}>{error}</div>}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setSettingsOpen(false)} disabled={saving}>Cancel</Button>
        <Button onClick={handleSave} variant="contained" disabled={saving}>Save</Button>
      </DialogActions>
    </Dialog>
  );
}; 