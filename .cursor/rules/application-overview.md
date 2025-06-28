# Wise0wl Launcher - Application Overview

## Purpose
Wise0wl Launcher is a Minecraft modpack launcher built with Tauri and React. It provides users with a streamlined way to discover, download, and launch Minecraft modpacks with integrated Microsoft authentication.

## Core Functionality

### Microsoft Authentication
- Secure Microsoft OAuth2 authentication flow
- Automatic token refresh and session management
- Support for Microsoft accounts and Xbox Live integration
- Secure storage of authentication tokens
- Multi-account support for different users

### Modpack Management
- Fetch modpack configurations from remote sources
- Display available modpacks with metadata (name, description, version, etc.)
- Support for multiple modpack sources and categories
- Local modpack installation and management

### Download System
- Download modpack files from various sources (CurseForge, Modrinth, direct URLs)
- Progress tracking and download management
- Resume interrupted downloads
- File integrity verification (checksums)

### Game Launching
- Launch Minecraft with selected modpacks using authenticated user
- Java version management and detection
- Memory allocation configuration
- Launch argument customization with authentication tokens
- Game logging and crash reporting

### User Experience
- Modern, intuitive interface built with React
- Cross-platform compatibility (Windows, macOS, Linux)
- Offline mode support for installed modpacks
- Settings and preferences management
- User profile management and switching
- Sidebar-driven navigation with modpack selection and user info
- Main view is a full-bleed modpack image with overlayed play button and progress bar
- Global navigation and dialog state managed via React Context
- Custom Material UI theme derived from Wise0wl logo for consistent branding

## Technical Architecture

### Frontend (React + TypeScript)
- Component-based UI architecture
- State management for authentication, modpacks, downloads, and settings
- Real-time progress updates and notifications
- Layout is optimized for desktop screens only; no mobile/tablet breakpoints
- Authentication flow UI components

### Backend (Rust + Tauri)
- Microsoft OAuth2 authentication handling
- Secure token storage and management
- File system operations and management
- Network requests and download handling
- Minecraft process management with authentication
- System integration and native APIs

### Data Management
- Secure storage for authentication tokens and user data
- Local storage for modpack metadata and settings
- Configuration files for modpack sources
- Download cache and temporary file management
- User preferences and launch configurations

## Security Considerations
- Secure OAuth2 implementation with proper token handling
- Encrypted storage of authentication credentials
- Validate all downloaded files and configurations
- Safe file system operations with proper permissions
- Input validation for all user-provided data
- Secure communication with Microsoft authentication servers

## Distribution
- Packaged as a standalone executable
- Auto-update functionality for the launcher itself
- Cross-platform builds for different operating systems
- User-friendly installation process
- Privacy-compliant authentication flow

## Desktop-Only Layout
- Use a permanent sidebar for navigation and user info
- Main content area features a full-bleed modpack view with overlayed actions
- No per-modpack changelog in the main view

## Platform
- Desktop-only: Windows, macOS, Linux. No mobile/tablet support.

## Custom Color Palette
- Use a custom color palette derived from the Wise0wl logo
- Ensure all sidebar and overlay text is visible in both light and dark modes

## Platform Awareness
- Always assume a desktop-only context (Windows, macOS, Linux)
- Do not suggest mobile/tablet layouts, breakpoints, or mobile-first design