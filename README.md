# NoBullFit App

## Development

For development, the app loads `http://localhost:4000` (your local Phoenix development server):

```bash
# Start the development version
npm run tauri:dev
```

## Production

For production builds, the app loads `https://nobull.fit`:

```bash
# Build for production
npm run tauri:build
```

## Build Scripts

- `npm run tauri:dev` - Development mode (localhost:4000)
- `npm run tauri:build` - Production build (nobull.fit)
- `npm run build` - Build web assets
- `npm run tauri` - Default Tauri command

## Features

- **File Import**: Native file picker for CSV imports
- **Custom User-Agent**: `NBFAPP/1.0 (Tauri)` for Phoenix detection
- **Security**: Proper CSP and capabilities configuration
- **Cross-platform**: Windows MSI, Linux AppImage/Deb support

## Configuration

The app uses separate configuration files:
- `src-tauri/tauri.dev.conf.json` - Development configuration
- `src-tauri/tauri.conf.json` - Production configuration