# Soplang for macOS

This directory contains the files needed to build Soplang for macOS, including the application bundle and disk image.

## Prerequisites

Before building Soplang for macOS, you need:

1. **macOS 10.13 or higher** - Older versions may work but are not tested
2. **Python 3.6 or higher** - Install via:
   ```
   # Using Homebrew (recommended)
   brew install python

   # Or download from Python.org
   # https://www.python.org/downloads/mac-osx/
   ```

3. **create-dmg** (optional, for creating disk image) - Install via:
   ```
   brew install create-dmg
   ```

## Building Soplang for macOS

Building Soplang on macOS is straightforward:

1. Navigate to the Soplang macos directory:
   ```bash
   cd soplang/macos
   ```

2. Make the build script executable (if needed):
   ```bash
   chmod +x build_macos.sh
   ```

3. Run the build script:
   ```bash
   ./build_macos.sh
   ```

The script will:
- Create a Python virtual environment
- Install all required dependencies
- Build the executable using PyInstaller
- Package it as a macOS application bundle
- Create a disk image for distribution (if create-dmg is installed)

## Build Output

After the build process completes successfully, you will find:

- **App Bundle**: `dist/Soplang.app`
- **Disk Image** (if create-dmg is installed): `macos/Soplang-<version>.dmg`

## Custom Icon

macOS applications require an `.icns` icon format. To create a custom icon:

1. Make the icon preparation script executable:
   ```bash
   chmod +x prepare_logos.sh
   ```

2. Run the icon preparation script:
   ```bash
   ./prepare_logos.sh
   ```

3. Follow the prompts to select or provide a path to your logo image
   - The script will use macOS built-in tools (sips and iconutil) to create the icon
   - It requires a high-resolution image (1024x1024 pixels recommended)
   - PNG format with transparency works best

## Installation

### From DMG

If you've created a DMG image:

1. Open the `.dmg` file
2. Drag `Soplang.app` to your Applications folder
3. Eject the disk image

### Manual Installation

You can also manually install the app bundle:

1. Navigate to `dist/` in Finder
2. Drag `Soplang.app` to your Applications folder or any other location

## File Associations

The build process automatically sets up file associations for `.sop` and `.so` files. After installation:

1. `.sop` and `.so` files will show the Soplang icon
2. Double-clicking these files will open them with Soplang
3. You can also use "Open With" to select Soplang for other file types

## Running Soplang

After installation, you can run Soplang in several ways:

1. Launch from Applications folder or Dock
2. Double-click any `.sop` or `.so` file
3. Open Terminal and run:
   ```bash
   /Applications/Soplang.app/Contents/MacOS/soplang
   ```

## Creating a Distributable Release

To create a release for distribution:

1. Build with create-dmg installed to get a proper disk image
2. (Optional) Sign the application with your Developer ID:
   ```bash
   codesign --force --sign "Developer ID Application: Your Name" dist/Soplang.app
   ```
3. (Optional) Notarize with Apple for improved security:
   ```bash
   xcrun altool --notarize-app --primary-bundle-id "org.soplang" --username "apple@id.com" --password "app-specific-password" --file "macos/Soplang-<version>.dmg"
   ```

## Troubleshooting

- **Build errors**: Make sure Python 3.6+ and pip are properly installed
- **Icon creation fails**: Ensure you're using a high-quality source image
- **App won't open**: Check Console.app for logs about why the app was prevented from opening
- **"App is damaged"**: This is a security feature of macOS - you may need to:
  - Open System Preferences → Security & Privacy
  - Click "Open Anyway" for Soplang
  - Or sign the app with your Developer ID
