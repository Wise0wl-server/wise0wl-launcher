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
- **Directory (tenant) ID**

### 3. Create Client Secret

1. Go to **Certificates & secrets**
2. Click **New client secret**
3. Add a description and choose expiration
4. **Important**: Copy the secret value immediately (you won't see it again)

### 4. Configure Permissions

1. Go to **API permissions**
2. Click **Add a permission**
3. Select **Xbox Live**
4. Choose **Delegated permissions**
5. Select:
   - `XboxLive.signin`
   - `XboxLive.ReadBasicProfile`
6. Click **Add permissions**
7. Click **Grant admin consent**

### 5. Update Application Code

1. Open `src-tauri/src/lib.rs`
2. Find the OAuth configuration section
3. Replace the placeholder values:
   ```rust
   const MICROSOFT_CLIENT_ID: &str = "your-actual-client-id";
   const MICROSOFT_CLIENT_SECRET: &str = "your-actual-client-secret";
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
