import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ModpackProvider } from './contexts/ModpackContext';
import { ThemeProvider } from './contexts/ThemeContext';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <ModpackProvider>
        <App />
      </ModpackProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
