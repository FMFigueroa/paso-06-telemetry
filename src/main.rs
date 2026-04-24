// ─── Paso 6: Telemetry — El device habla primero ───
//
// El firmware adquiere la capacidad de reportar su propio estado de
// salud periódicamente, sin esperar a que nadie pregunte. Cada 60 s
// construye un TelemetryReport con heap libre, uptime, y snapshot del
// LightState, y lo envía via WS.
//
// Módulo nuevo: telemetry
// ws_client extendido con OutgoingMessage::Telemetry(...)

mod led;
mod light_state;
mod provisioning;
mod secure_storage;
mod telemetry;
mod wifi;
mod ws_client;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

#[allow(unused_imports)]
use esp_idf_svc::sys as _;

use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use led::LedController;
use light_state::{LightState, Mode};
use secure_storage::SecureStorage;
use telemetry::TelemetryReport;
use ws_client::{OutgoingMessage, WsClient};

const BRIGHTNESS_STEPS: &[u8] = &[0, 25, 50, 75, 100, 75, 50, 25];
const LOOP_TICK_MS: u32 = 500;

/// Intervalo entre reportes de telemetría. 60 s es un default razonable
/// para IoT consumer — frecuente para detectar problemas, poco enough
/// para no saturar el backend con miles de devices.
const TELEMETRY_INTERVAL: Duration = Duration::from_secs(60);

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("paso-06-telemetry");

    if let Err(e) = run() {
        error!("Error fatal: {:?}", e);
        std::thread::sleep(Duration::from_secs(10));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    }
}

fn run() -> anyhow::Result<()> {
    // Capturamos el boot time para calcular uptime de la telemetría.
    // Instant::now() monotonic — no afectado por cambios del reloj.
    let boot_time = Instant::now();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs_partition = EspDefaultNvsPartition::take()?;

    let led_controller = LedController::new(peripherals.rmt.channel0, peripherals.pins.gpio2)?;
    let led = Arc::new(Mutex::new(led_controller));

    let storage = SecureStorage::new(nvs_partition.clone())?;
    let storage = Arc::new(Mutex::new(storage));

    let is_provisioned = { storage.lock().unwrap().is_provisioned()? };
    if !is_provisioned {
        warn!("Device not provisioned!");
        info!("Connect to 'Leonobitech-Setup' → http://192.168.4.1");
        provisioning::start_provisioning(peripherals.modem, sysloop, storage)?;
        return Ok(());
    }

    let credentials = { storage.lock().unwrap().load_credentials()? };
    let device_id = credentials.device_id.clone();

    info!(
        "Device ID: {} — Connecting to WiFi: {}",
        device_id, credentials.wifi_ssid
    );
    let _wifi = wifi::connect(
        &credentials.wifi_ssid,
        &credentials.wifi_password,
        peripherals.modem,
        sysloop,
    )?;
    info!("WiFi connected!");
    drop(credentials);

    let light_state = Arc::new(Mutex::new(LightState::default()));

    let ws = WsClient::new(light_state.clone())?;
    ws.send(OutgoingMessage::Hello {
        device_id: device_id.clone(),
    })?;

    info!(
        "Entering main loop — telemetry every {} s",
        TELEMETRY_INTERVAL.as_secs()
    );

    let mut step_idx: usize = 0;
    let mut last_manual_intensity: u8 = 255;
    let mut next_telemetry = Instant::now() + TELEMETRY_INTERVAL;

    loop {
        // ─── Dispatch por Mode (heredado de paso-05) ───

        let snapshot = { *light_state.lock().unwrap() };

        match snapshot.mode {
            Mode::Auto => {
                let step = BRIGHTNESS_STEPS[step_idx];
                led.lock().unwrap().set_brightness(step)?;
                light_state.lock().unwrap().intensity = step;
                step_idx = (step_idx + 1) % BRIGHTNESS_STEPS.len();
                last_manual_intensity = 255;
            }
            Mode::Manual => {
                if snapshot.intensity != last_manual_intensity {
                    led.lock().unwrap().set_brightness(snapshot.intensity)?;
                    last_manual_intensity = snapshot.intensity;
                }
            }
        }

        // ─── Tick de telemetría (cada 60 s) ───
        //
        // En vez de usar un thread separado para la telemetría, la metemos
        // en el main loop. Ventaja: no hay otro thread competiendo por
        // los locks, el orden de operaciones es determinista. Desventaja:
        // el intervalo real es múltiplo del LOOP_TICK_MS (500 ms) — si
        // pusiéramos 1 s entre telemetrías, sería irrealizable acá.

        if Instant::now() >= next_telemetry {
            let mode_str = match snapshot.mode {
                Mode::Auto => "auto",
                Mode::Manual => "manual",
            };

            let report = TelemetryReport::new(boot_time)
                .with_heap()
                .with_light_state(snapshot.intensity, mode_str);

            info!(
                "Telemetry: uptime={}s heap={:?} intensity={} mode={}",
                report.uptime_secs, report.heap_free_bytes, report.intensity, report.mode
            );

            if let Err(e) = ws.send(OutgoingMessage::Telemetry(report)) {
                warn!("Failed to enqueue telemetry: {:?}", e);
            }

            next_telemetry = Instant::now() + TELEMETRY_INTERVAL;
        }

        FreeRtos::delay_ms(LOOP_TICK_MS);
    }
}
