# PadConnectReceiver

[![Discord](https://img.shields.io/discord/1496412688685858846?label=&logo=discord&logoColor=ffffff&color=5865F2&labelColor=404EED)](https://discord.gg/BrMAZbEyXs)

> **Desktop receiver for PadConnect** - turns UDP input from your phone into a real virtual gamepad

PadConnectReceiver is the Desktop side companion to **[PadConnect](https://github.com/Ishan09811/PadConnect)**. It listens for low latency controller input streamed from the PadConnect Android app and exposes it to Desktop (and games) as a real virtual controller, using **ViGEm** on windows and **uinput** on linux.

This is one half of a two-part project:

- **[PadConnect](https://github.com/Ishan09811/PadConnect)** -> Android / client app (virtual controller UI)
- **PadConnectReceiver** -> Desktop / receiver app (creates the virtual controller) *(this repo)*

---

## How it works

```
[ Android Phone ] -- UDP --> [ PadConnectReceiver (Windows) ] --> [ ViGEm ] --> Game
```

1. **[PadConnect](https://github.com/Ishan09811/PadConnect) (Android)** renders a virtual controller, captures input, and streams it over UDP on the local WiFi network.
2. **PadConnectReceiver (Desktop)** listens for those UDP packets, executes the controller states, which exposes a virtual Xbox 360 (DualShock4 coming soon) controller to the OS.

Games see it as a *real* controller.

---

## Features

- Low-latency **UDP** input receiving
- **Xbox 360** virtual controller support (working) **DualShock4** support planned
- Built with **Rust**
- Works over local WiFi, no internet required
- Pairs with the [PadConnect](https://github.com/Ishan09811/PadConnect) Android app

---

## Requirements

- Windows (10 / 11) / Linux
- **ViGEmBus Driver** installed **(only required for windows users)**
- The [PadConnect](https://github.com/Ishan09811/PadConnect) Android app running on the same local WiFi network

---

## Getting Started

### 1. Install ViGEm (only needed for windows users)

Download and install **ViGEmBus** from the [official ViGEm GitHub release](https://github.com/ViGEm/ViGEmBus/releases), then **reboot**.

### 2. Run PadConnectReceiver

```
PadConnectReceiver.exe (windows)
PadConnectReceiver.AppImage (linux)
```

This starts listening for UDP input and creates a virtual controller.

### 3. Connect from PadConnect (Android)

- Install [PadConnect](https://github.com/Ishan09811/PadConnect) on your phone
- Create a controller layout
- start playing

> **Version compatibility:** PadConnectReceiver and PadConnect releases are paired, check each release's notes for the minimum required companion version before pairing an older client/receiver combination.

---

# Download
You can download stable builds from the Releases tab, or you can download the latest build from the tables below.

Development Builds:
|Platform|ABI|Download|
|--------|------------|--------|
|Windows|x86_64|[Windows Executable](https://nightly.link/PadConnectApp/PadConnectReceiver/workflows/build/master/PadConnectReceiver-windows-x86_64.zip)|
|Windows|arm64|[Windows Executable](https://nightly.link/PadConnectApp/PadConnectReceiver/workflows/build/master/PadConnectReceiver-windows-arm64.zip)|
|Linux|x86_64|[Linux AppImage](https://nightly.link/PadConnectApp/PadConnectReceiver/workflows/build/master/PadConnectReceiver-linux-x86_64.zip)|
|Linux|arm64|[Linux AppImage](https://nightly.link/PadConnectApp/PadConnectReceiver/workflows/build/master/PadConnectReceiver-linux-arm64.zip)|

## Supported Inputs

- Buttons: A / B / X / Y
- Shoulder buttons
- Triggers
- Analog Sticks
- Dpad

## Notes

- Works best on local WiFi
- Firewall may need to allow the UDP port used for input streaming
- ViGEm is only required for Windows

---

## Credits

- **ViGEm** — Virtual Gamepad Emulation Framework for windows support

---

## License

This project is licensed under the GNU General Public License v3.0 ([GPL-3.0-only](LICENSE)).

---

> Built for low latency, simplicity, and fun.
