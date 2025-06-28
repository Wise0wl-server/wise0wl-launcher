import React, { createContext, useContext, useState } from 'react';

interface UIContextType {
  settingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;
  changelogOpen: boolean;
  setChangelogOpen: (open: boolean) => void;
}

const UIContext = createContext<UIContextType | undefined>(undefined);

export const useUI = () => {
  const context = useContext(UIContext);
  if (!context) throw new Error('useUI must be used within a UIProvider');
  return context;
};

export const UIProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [changelogOpen, setChangelogOpen] = useState(false);

  return (
    <UIContext.Provider value={{
      settingsOpen,
      setSettingsOpen,
      changelogOpen,
      setChangelogOpen,
    }}>
      {children}
    </UIContext.Provider>
  );
}; 