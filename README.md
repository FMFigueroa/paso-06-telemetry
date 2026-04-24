# Rust Embedded desde Cero

## paso-06-telemetry

[![ESP32 CI](https://github.com/FMFigueroa/paso-06-telemetry/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/FMFigueroa/paso-06-telemetry/actions/workflows/rust_ci.yml)

<p align="center">
  <img src="docs/rust-board.png" alt="ESP32-C3-DevKit-RUST-1" width="600">
</p>

Reportes periódicos de salud — el firmware habla primero. Cada 60 s envía un `TelemetryReport` con heap libre (FFI al SDK de C), uptime del sistema, y snapshot del `LightState`. Introduce builder pattern, `#[serde(skip_serializing_if = "Option::is_none")]` para campos opcionales, y el patrón "trabajo periódico en main loop via contador `Instant`".

## Qué hace este paso

Todo lo de paso-05 + cada 60 s emite `{"type":"Telemetry","uptime_secs":3600,"heap_free_bytes":180432,"mode":"auto","intensity":75}` via WS.

## Ver la telemetría en vivo

```bash
websocat wss://ws.postman-echo.com/raw
```

Desde tu terminal conectada al mismo echo server, vas a ver llegar el TelemetryReport del device cada 60 s (el echo lo devuelve a todos los clientes conectados — incluyendote).

## Estructura

```
src/
  main.rs                  # Loop con contador de telemetría + dispatch por Mode
  telemetry.rs             # TelemetryReport con builder (NUEVO)
  ws_client.rs             # + variant Telemetry(TelemetryReport)
  light_state.rs           # Heredado intacto
  led.rs, wifi.rs,         # Heredados intactos
  secure_storage.rs,
  provisioning.rs
```

## Dependencias

Paso-06 no agrega deps nuevas.

## Documentacion

<a href="https://discord.gg/dYrqe9HZfz">
  <img alt="Discord" width="35px" src="https://img.icons8.com/external-justicon-lineal-color-justicon/64/external-discord-social-media-justicon-lineal-color-justicon.png"/>
</a>&ensp;
<a href="https://discord.gg/dYrqe9HZfz"><strong>Unirse al servidor — Curso Rust Embedded</strong></a>

## Roadmap

> Este repo es el **Paso 6** del curso **Rust Embedded desde Cero**.

- [Paso 1 — Scaffold del proyecto](https://github.com/FMFigueroa/paso-01-scaffold)
- [Paso 2 — WiFi Station](https://github.com/FMFigueroa/paso-02-wifi-station)
- [Paso 3 — LED PWM](https://github.com/FMFigueroa/paso-03-led-pwm)
- [Paso 4 — WebSocket Client](https://github.com/FMFigueroa/paso-04-websocket)
- [Paso 5 — Light State Management](https://github.com/FMFigueroa/paso-05-light-state)
- **[Paso 6 — Telemetria](https://github.com/FMFigueroa/paso-06-telemetry)** ← _este repo_
- [Paso 7 — Time Sync (SNTP)](https://github.com/FMFigueroa/paso-07-time-sync)
- [Paso 8 — Schedule & Auto Mode](https://github.com/FMFigueroa/paso-08-schedule)
- [Paso 9 — Concurrencia & Watchdog](https://github.com/FMFigueroa/paso-09-watchdog)


## Licencia

[MIT](LICENSE)
