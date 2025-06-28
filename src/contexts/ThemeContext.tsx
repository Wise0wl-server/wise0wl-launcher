import React, { createContext, useContext, useState, useEffect } from 'react';
import { ThemeProvider as MuiThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

interface ThemeContextType {
  darkMode: boolean;
  toggleDarkMode: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [darkMode, setDarkMode] = useState(() => {
    // Check localStorage for saved preference, default to light mode
    const saved = localStorage.getItem('darkMode');
    return saved ? JSON.parse(saved) : false;
  });

  useEffect(() => {
    // Save preference to localStorage whenever it changes
    localStorage.setItem('darkMode', JSON.stringify(darkMode));
  }, [darkMode]);

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
  };

  const theme = createTheme({
    palette: {
      mode: darkMode ? 'dark' : 'light',
      primary: {
        main: '#636469', // Slate Gray
        contrastText: darkMode ? '#F6F1D0' : '#000000', // Light Cream for contrast in dark, black in light
      },
      secondary: {
        main: '#ACA896', // Warm Gray
        contrastText: darkMode ? '#F6F1D0' : '#000000',
      },
      background: {
        default: darkMode ? '#000000' : '#F6F1D0',
        paper: darkMode ? '#232323' : '#ffffff', // Slightly lighter than black for paper in dark mode
      },
      text: {
        primary: darkMode ? '#F6F1D0' : '#000000',
        secondary: darkMode ? '#F6F1D0' : '#636469', // Use light text in dark mode for both
      },
      info: {
        main: '#9B9885',
        contrastText: darkMode ? '#F6F1D0' : '#000000',
      },
      warning: {
        main: '#F6F1D0', // Use cream as a highlight
        contrastText: '#636469',
      },
    },
    typography: {
      fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
      h4: {
        fontWeight: 600,
      },
      h5: {
        fontWeight: 600,
      },
      h6: {
        fontWeight: 600,
      },
    },
    components: {
      MuiButton: {
        styleOverrides: {
          root: {
            textTransform: 'none',
            borderRadius: 8,
            fontWeight: 600,
          },
          containedPrimary: {
            backgroundColor: darkMode ? '#636469' : '#9B9885', // Use accent in light mode
            color: darkMode ? '#F6F1D0' : '#000000',
            '&:hover': {
              backgroundColor: darkMode ? '#ACA896' : '#636469',
            },
          },
          outlinedPrimary: {
            borderColor: darkMode ? '#F6F1D0' : '#636469',
            color: darkMode ? '#F6F1D0' : '#636469',
            backgroundColor: darkMode ? 'rgba(246,241,208,0.08)' : 'rgba(155,152,133,0.08)',
            '&:hover': {
              borderColor: darkMode ? '#9B9885' : '#9B9885',
              backgroundColor: darkMode ? 'rgba(155,152,133,0.16)' : 'rgba(99,100,105,0.16)',
            },
          },
        },
      },
      MuiCard: {
        styleOverrides: {
          root: {
            borderRadius: 12,
            boxShadow: darkMode 
              ? '0 2px 8px rgba(0,0,0,0.5)' 
              : '0 2px 8px rgba(0,0,0,0.1)',
            backgroundColor: darkMode ? '#232323' : '#ffffff',
            color: darkMode ? '#F6F1D0' : '#000000',
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            borderRadius: 12,
            backgroundColor: darkMode ? '#232323' : '#ffffff',
            color: darkMode ? '#F6F1D0' : '#000000',
          },
        },
      },
      MuiDrawer: {
        styleOverrides: {
          paper: {
            backgroundColor: darkMode ? '#636469' : '#ACA896',
            borderRight: `1px solid ${darkMode ? '#000000' : '#F6F1D0'}`,
            color: darkMode ? '#F6F1D0' : '#000000',
          },
        },
      },
      MuiAppBar: {
        styleOverrides: {
          root: {
            backgroundColor: darkMode ? '#000000' : '#636469',
            color: darkMode ? '#F6F1D0' : '#F6F1D0',
          },
        },
      },
    },
  });

  const contextValue: ThemeContextType = {
    darkMode,
    toggleDarkMode,
  };

  return (
    <ThemeContext.Provider value={contextValue}>
      <MuiThemeProvider theme={theme}>
        <CssBaseline />
        {children}
      </MuiThemeProvider>
    </ThemeContext.Provider>
  );
}; 