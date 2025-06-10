# File Explorer

A modern, cross-platform File Explorer built with [Tauri](https://tauri.app/), featuring a React (Vite) frontend and a Rust backend. This application allows you to browse, search, and open files and folders on your system with a clean, desktop-like interface.

## Features
- Browse files and folders with a responsive UI
- Search files by name and extension
- Open files and folders directly from the app
- View file details: name, directory, type, size, and last modified date
- Breadcrumb navigation and quick actions (Home, Up, Refresh)
- Fast and lightweight, thanks to Tauri and Rust

## Screenshots
<!-- Add screenshots here -->

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) (v16 or newer)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites/):
  ```sh
  cargo install tauri-cli
  ```

### Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/YOUR-USERNAME/YOUR-REPO.git
   cd YOUR-REPO
   ```
2. Install dependencies:
   ```sh
   npm install
   ```

### Development
To run the app in development mode:
```sh
npm run tauri dev
```

### Build (Create .exe)
To build a standalone executable for Windows:
```sh
npm run tauri build
```
- The `.exe` will be in `src-tauri/target/release/`
- Installers (optional) are in `src-tauri/target/release/bundle/`

### Run the .exe
Double-click the generated `.exe` file to launch the app without needing Node.js or npm.

## Technologies Used
- [Tauri](https://tauri.app/)
- [React](https://react.dev/)
- [Vite](https://vitejs.dev/)
- [Rust](https://www.rust-lang.org/)
- [Material UI](https://mui.com/)

## License
This project is licensed under the MIT License.
