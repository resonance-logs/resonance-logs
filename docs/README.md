# Backend Documentation

Welcome to the backend documentation for the Resonance Logs application. This documentation provides a comprehensive overview of the backend architecture, modules, and implementation details.

## Overview

The backend is built with Rust using the [Tauri](https://tauri.app/) framework, which allows for the creation of cross-platform desktop applications with a web frontend. The backend is responsible for the following key functionalities:

- **Real-time Packet Capture:** Capturing and processing network packets to extract relevant game data.
- **Data Persistence:** Storing and managing game data in a local SQLite database.
- **Real-time Event Processing:** Processing game events and emitting them to the frontend for real-time display.
- **API for Frontend:** Providing a set of commands that the frontend can invoke to interact with the backend.

## Modules

The backend is organized into the following modules:

- **`live`:** Handles real-time event processing.
- **`database`:** Manages the SQLite database.
- **`packets`:** Responsible for packet capture and processing.
- **`blueprotobuf-lib`:** Contains the protobuf definitions for the game's data protocol.

For more detailed information on each module, please refer to the respective documentation files:

- [Application Structure](./APP_STRUCTURE.md)
- [Live Module](./LIVE_MODULE.md)
- [Database Module](./DATABASE_MODULE.md)
- [Packets Module](./PACKETS_MODULE.md)
- [Blueprotobuf Lib](./BLUEPROTOBUF_LIB.md)
