# eGCS

This is a simple ground control station built for modern PX4 drones, written in Rust with egui.

## Demo Video
https://github.com/user-attachments/assets/cacfab63-7f04-4396-9ae7-a0bd5136023a

## Features
- Real-time MAVLink telemetry
- Flight mode control
- Live satellite map with vehicle tracking
- Go-to-position commanding

## Architecture

eGCS is designed to be quick, responsive and **never** hold the main UI thread, so that users always see the most up-to-date information, and their commands get sent as soon as possible. To accomplish this, the egui frontend is separated from all MAVLink I/O by running the connection on a dedicated thread hosting a tokio runtime. On this backend thread, a tokio::select! loop concurrently handles inbound telemetry, outbound commands, and a 1 Hz GCS heartbeat. Communication to/from the tokio runtime thread and the UI is done using channels.

Three channels currently cross the thread boundary. First is an mpsc channel that handles commands from the UI to the backend (e.g. connect, disconnect, arm, takeoff). Next is an mpsc channel that reports connection status back to the UI (specifically link state, for now). Finally, there's a watch channel that reports the latest telemetry snapshot.

All of this together creates an architecture that can be used to fly a drone in real time.

### File Tree

```
  egcs/
  ├─ assets/                  # app icon and static resources
  ├─ src/
  │  ├─ connection/           # async MAVLink link, owns all I/O
  │  │  ├─ mod.rs             # Command / VehicleCommand / ConnStatus types, GCS header
  │  │  ├─ link.rs            # tokio task: connection lifecycle + tokio::select! event loop (recv telemetry, send commands, 1 Hz heartbeat)
  │  │  └─ protocol.rs        # builds outbound MAVLink messages (arm, takeoff, reposition, set-mode, stream/param requests)
  │  ├─ telemetry/            # vehicle state, no I/O
  │  │  ├─ mod.rs             # Telemetry snapshot struct + update() ingesting inbound messages
  │  │  ├─ vehicle_state.rs   # derived status: armed, flying, landing, selectable modes
  │  │  ├─ position.rs        # position/velocity accessors with unit conversions
  │  │  └─ flight_modes.rs    # PX4 standard vs custom mode handling
  │  ├─ ui/                   # egui views, render only
  │  │  ├─ mod.rs
  │  │  ├─ app.rs             # top-level app, panels, command dispatch
  │  │  ├─ map.rs             # satellite map + vehicle marker (walkers)
  │  │  └─ theme.rs           # shared widgets and styling
  │  ├─ main.rs               # native entry points
  │  └─ lib.rs                # crate root

```

## Known Limitations
- **Single Vehicle Support:** Built specifically for individual PX4 instances. Multi-vehicle routing not supported yet.
- **No Command Failure Feedback:** Commands are dispatched asynchronously over MAVLink, but the UI does not currently track MAVLink `COMMAND_ACK` timeouts or display execution failure alerts to the user.
- **Missing Heartbeat Timeout & Connection State Machine:** Once connected, the backend lacks a heartbeat timeout monitor to detect lost telemetry streams. Additionally, there is no intermediate "Communication Lost" state to maintain the UI layout and attempt auto-reconnection before requiring a manual disconnect.
- **PX4 1.15+ Mode Requirements:** Relies on the modern `AVAILABLE_MODES` MAVLink interface, requiring PX4 v1.15 or newer (legacy custom mode mapping is omitted for scope).
- **PX4-Specific Dialect:** Tailored exclusively to PX4 flight stack conventions; ArduPilot custom mode flags and command sets are not supported.

## Future Scope
- [ ] **Parameter Management:** Full fetch, search, edit, and onboard parameter syncing (`PARAM_EXT_REQUEST_READ` / `PARAM_SET`).
- [ ] **Attitude & Compass Widget:** Artificial horizon, pitch/roll indicators, and heading compass tape.
- [ ] **Manual Control / Joystick Integration:** Direct manual piloting support via gamepad/joystick mapping (`MANUAL_CONTROL` MAVLink messages).
- [ ] **Mission Planning & Waypoints:** Interactive map drawing for waypoints, route uploads, and mission execution (`MAV_CMD_NAV_WAYPOINT`).
- [ ] **Command Retry & Timeout Handling:** Track `COMMAND_ACK` responses to surface execution alerts and auto-retry dropped commands.
- [ ] **Link Health Watchdog:** Implement a watchdog timer to transition connection state (`Connected` → `Comm Lost` → `Reconnecting`) without wiping UI context.

## Build & Run (Native Desktop)

### Install Dependencies (Linux)
`sudo apt install libxcb-render0-dev libxkbcommon-dev`

### Build
`cargo build`

### Run
`cargo run`

To test with a simulated UAV, follow the [**PX4 Simulation guide**](https://docs.px4.io/main/en/simulation/) to setup and run a PX4 SITL gazebo simulation locally and connect to it using eGCS via `udpin:0.0.0.0:14550`
