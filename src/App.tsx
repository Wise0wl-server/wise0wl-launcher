import { MainLayout } from './components/Layout/MainLayout';
import { HomePage } from './components/Pages/HomePage';
import { ThemeProvider } from './contexts/ThemeContext';
import { ModpackProvider } from './contexts/ModpackContext';
import { UIProvider } from './contexts/UIContext';
import { SettingsDialog } from './components/Pages/SettingsDialog';

function App() {
  return (
    <ThemeProvider>
      <ModpackProvider>
        <UIProvider>
          <MainLayout>
            <HomePage />
          </MainLayout>
          <SettingsDialog />
        </UIProvider>
      </ModpackProvider>
    </ThemeProvider>
  );
}

export default App;
