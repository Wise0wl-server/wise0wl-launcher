# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# Wise0wl Launcher

A Minecraft launcher with modpack support built with Tauri and React.

## Features

- Minecraft modpack management
- Microsoft OAuth authentication
- Group-based access control
- Auto-downloader for Fabric, Forge, and NeoForge
- Cross-platform support (Windows, macOS, Linux)

## Environment Setup

### 1. Configure Environment Variables

1. Copy the example environment file:
   ```bash
   cd src-tauri
   cp .env.example .env
   ```

2. Edit the `.env` file and set your configuration:
   ```env
   MICROSOFT_CLIENT_ID=your-actual-client-id
   OAUTH_REDIRECT_URI=wise0wl-oauth://callback
   OAUTH_SCOPES=XboxLive.signin offline_access
   ```

**Important**: Never commit your `.env` file to version control. It's already included in `.gitignore`.

## Microsoft OAuth Setup

To enable Microsoft authentication, you need to register an Azure AD application:

### 1. Create Azure AD Application

1. Go to [Azure Portal](https://portal.azure.com)
2. Navigate to **Azure Active Directory** → **App registrations**
3. Click **New registration**
4. Fill in the details:
   - **Name**: "Wise0wl Launcher"
   - **Supported account types**: "Accounts in any organizational directory and personal Microsoft accounts"
   - **Redirect URI**: 
     - Type: Web
     - URI: `http://localhost:1420` (for development)
     - Type: Custom
     - URI: `wise0wl-oauth://callback` (for production)

### 2. Get Application Credentials

After registration, note down:
- **Application (client) ID**

### 3. Configure Permissions

1. Go to **API permissions**
2. Click **Add a permission**
3. Select **Xbox Live**
4. Choose **Delegated permissions**
5. Select:
   - `XboxLive.signin`
   - `XboxLive.ReadBasicProfile`
6. Click **Add permissions**
7. Click **Grant admin consent**

### 4. Update Environment Configuration

1. Open `src-tauri/.env`
2. Replace `YOUR_MICROSOFT_CLIENT_ID_HERE` with your actual client ID:
   ```env
   MICROSOFT_CLIENT_ID=your-actual-client-id
   ```

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run tauri dev
```

## Building

```bash
# Build for production
npm run tauri build
```

## License

[Add your license here]
