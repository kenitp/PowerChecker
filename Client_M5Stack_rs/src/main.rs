// Power Checker Client
// Rust + Slint rewrite of the original C++ / PlatformIO project.
// Board-specific hardware is in the `board` module.

#![no_std]
#![no_main]

extern crate alloc;

mod board;
mod network;
mod power;
mod sdcard;
mod time_utils;

use alloc::{boxed::Box, format, rc::Rc};
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU8, Ordering};

use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
    clock::CpuClock,
    peripherals::Peripherals,
    rng::Rng,
    time::Instant,
    timer::timg::TimerGroup,
};
use esp_println::logger::init_logger_from_env;
use log::{debug, error, info, warn};

use slint::platform::WindowEvent;
use smoltcp::wire::Ipv4Address;

use board::{DrawBuffer, DISPLAY_HEIGHT, DISPLAY_WIDTH, HEAP_SIZE};
use network::NetworkState;
use power::{watts_to_level, watts_to_ratio};
use time_utils::{current_time, DOW_STR};

// ── Build-time WiFi / server config ──────────────────────────────────────────
const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");
const POWER_CHECKER_URL: &str = env!("POWER_CHECKER_URL");

// ── NTP ──────────────────────────────────────────────────────────────────────
const NTP_HOST_IP: Ipv4Address = Ipv4Address::new(133, 243, 238, 164); // ntp.nict.jp
const NTP_PORT: u16 = 123;
const JST_OFFSET_SECS: i32 = 9 * 3600;

const POWER_FETCH_INTERVAL_MS: u64 = 30_000;
const NTP_SYNC_INTERVAL_MS: u64 = 3_600_000;
const NTP_RETRY_INTERVAL_MS: u64 = 30_000;
const WIFI_RETRY_INTERVAL_MS: u64 = 15_000;

// ── Shared state ─────────────────────────────────────────────────────────────
pub(crate) static POWER_WATTS: AtomicU32 = AtomicU32::new(0);
pub(crate) static POWER_CENTI_A: AtomicU32 = AtomicU32::new(0);
pub(crate) static WIFI_CONNECTED: AtomicBool = AtomicBool::new(false);
pub(crate) static DATA_VALID: AtomicBool = AtomicBool::new(false);
pub(crate) static FORCE_REFRESH: AtomicBool = AtomicBool::new(false);
pub(crate) static NTP_EPOCH_SECS: AtomicI32 = AtomicI32::new(0);
pub(crate) static NTP_SYNC_MS: AtomicU32 = AtomicU32::new(0);

static PHOTO_NEXT: AtomicBool = AtomicBool::new(false);
static PHOTO_PREV: AtomicBool = AtomicBool::new(false);

const MODE_POWER: u8 = 0;
const MODE_CLOCK: u8 = 1;
const MODE_PHOTO: u8 = 2;
static CURRENT_MODE: AtomicU8 = AtomicU8::new(MODE_POWER);
static SD_OK: AtomicBool = AtomicBool::new(false);
static SD_FILE_COUNT: AtomicU32 = AtomicU32::new(0);

esp_app_desc!();

slint::include_modules!();

// ── Platform backend ─────────────────────────────────────────────────────────

struct EspBackend {
    window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
    peripherals: RefCell<Option<Peripherals>>,
}

impl slint::platform::Platform for EspBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(Instant::now().duration_since_epoch().as_millis())
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        self.run_event_loop_impl()
    }
}

impl EspBackend {
    fn run_event_loop_impl(&self) -> Result<(), slint::PlatformError> {
        let peripherals = self
            .peripherals
            .borrow_mut()
            .take()
            .expect("peripherals already taken");

        let hw = board::init_hardware(peripherals);

        let size = slint::PhysicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
        self.window.borrow().as_ref().unwrap().set_size(size);

        // ── SD card ───────────────────────────────────────────────────────────
        let mut sd: Option<sdcard::SdReader> = match sdcard::SdReader::new(hw.sd_spi) {
            Ok(mut reader) => {
                reader.scan_all();
                let count = reader.photo_count();
                SD_OK.store(true, Ordering::Relaxed);
                SD_FILE_COUNT.store(count as u32, Ordering::Relaxed);
                Some(reader)
            }
            Err(e) => {
                warn!("SD card unavailable: {:?}", e);
                SD_OK.store(false, Ordering::Relaxed);
                None
            }
        };
        let mut photo_idx: usize = 0;
        let mut photo_needs_draw = false;
        let mut photo_on_screen = false;
        let mut prev_mode: u8 = MODE_POWER;
        // Track last power level to know when to redraw the power overlay image
        let mut power_img_level: Option<u8> = None;

        // ── WiFi & smoltcp setup ─────────────────────────────────────────────
        let timg1 = TimerGroup::new(hw.wifi_timer_peripheral);
        let rng = Rng::new(hw.rng_peripheral);

        let mut net: Option<NetworkState> = esp_wifi::init(timg1.timer0, rng)
            .map_err(|e| {
                error!("esp-wifi init: {:?}", e);
            })
            .ok()
            .and_then(|init| {
                let init_ref: &'static _ = Box::leak(Box::new(init));
                esp_wifi::wifi::new(init_ref, hw.wifi_peripheral)
                    .map_err(|e| {
                        error!("wifi::new: {:?}", e);
                    })
                    .ok()
            })
            .and_then(|(controller, ifaces)| {
                let esp_wifi::wifi::Interfaces { sta, ap, .. } = ifaces;
                NetworkState::connect(sta, ap, controller)
                    .map_err(|e| {
                        warn!("WiFi connect: {:?}", e);
                    })
                    .ok()
            });
        let mut last_wifi_retry_ms: u64 = 0;

        // ── Line-buffer renderer ─────────────────────────────────────────────
        let mut buf_provider = DrawBuffer {
            display: hw.display,
            buffer: &mut [slint::platform::software_renderer::Rgb565Pixel(0);
                DISPLAY_WIDTH as usize],
        };

        let mut prev_btn = [false; 3];
        let mut last_fetch_ms: u64 = 0;
        let mut last_ntp_ms: u64 = 0;
        let mut last_ntp_attempt_ms: u64 = 0;

        // ── Main event loop ──────────────────────────────────────────────────
        loop {
            slint::platform::update_timers_and_animations();

            let now_ms = Instant::now().duration_since_epoch().as_millis();

            // ── Mode transition detection ─────────────────────────────────────
            let cur_mode = CURRENT_MODE.load(Ordering::Relaxed);
            if cur_mode != prev_mode {
                if cur_mode == MODE_PHOTO {
                    let sd_has_files = sd.as_ref().map_or(false, |s| s.photo_count() > 0);
                    if sd_has_files {
                        photo_needs_draw = true;
                    }
                }
                if prev_mode == MODE_PHOTO {
                    photo_on_screen = false;
                }
                // Force power overlay redraw when switching back to POWER mode
                if cur_mode == MODE_POWER {
                    power_img_level = None;
                }
                prev_mode = cur_mode;
            }

            // ── Button input ──────────────────────────────────────────────────
            let btns = [hw.btn_a.is_low(), hw.btn_b.is_low(), hw.btn_c.is_low()];
            if let Some(window) = self.window.borrow().clone() {
                for i in 0..3 {
                    let key: slint::SharedString = match i {
                        0 => slint::platform::Key::LeftArrow.into(),
                        1 => slint::platform::Key::Return.into(),
                        _ => slint::platform::Key::RightArrow.into(),
                    };
                    if btns[i] && !prev_btn[i] {
                        window
                            .try_dispatch_event(WindowEvent::KeyPressed { text: key })
                            .ok();
                    } else if !btns[i] && prev_btn[i] {
                        let key2: slint::SharedString = match i {
                            0 => slint::platform::Key::LeftArrow.into(),
                            1 => slint::platform::Key::Return.into(),
                            _ => slint::platform::Key::RightArrow.into(),
                        };
                        window
                            .try_dispatch_event(WindowEvent::KeyReleased { text: key2 })
                            .ok();
                    }
                }
            }
            prev_btn = btns;

            // ── Photo navigation ──────────────────────────────────────────────
            if let Some(ref sd_reader) = sd {
                let count = sd_reader.photo_count();
                if count > 0 {
                    if PHOTO_NEXT.swap(false, Ordering::Relaxed) {
                        photo_idx = (photo_idx + 1) % count;
                        photo_needs_draw = true;
                    }
                    if PHOTO_PREV.swap(false, Ordering::Relaxed) {
                        photo_idx = if photo_idx == 0 {
                            count - 1
                        } else {
                            photo_idx - 1
                        };
                        photo_needs_draw = true;
                    }
                }
            }

            // ── Network polling ───────────────────────────────────────────────
            if let Some(ref mut ns) = net {
                ns.poll(now_ms);
            }

            let force = FORCE_REFRESH.swap(false, Ordering::Relaxed);
            if force || (now_ms - last_fetch_ms) >= POWER_FETCH_INTERVAL_MS {
                if let Some(ref mut ns) = net {
                    if ns.ip_assigned {
                        debug!("[Main] Fetching power (now_ms={})", now_ms);
                        match ns.http_get_power(now_ms) {
                            Ok((w, ca)) => {
                                POWER_WATTS.store(w, Ordering::Relaxed);
                                POWER_CENTI_A.store(ca, Ordering::Relaxed);
                                DATA_VALID.store(true, Ordering::Relaxed);
                                info!("[Power] {}W  {}.{:02}A", w, ca / 100, ca % 100);
                            }
                            Err(e) => warn!("[Power] Fetch failed: {:?}", e),
                        }
                    } else {
                        debug!("[Main] No IP yet, trying reconnect");
                        ns.try_reconnect();
                    }
                } else if now_ms - last_wifi_retry_ms >= WIFI_RETRY_INTERVAL_MS {
                    warn!("[Main] WiFi not initialized");
                    last_wifi_retry_ms = now_ms;
                }
                last_fetch_ms = now_ms;
            }

            if let Some(ref mut ns) = net {
                let ntp_due = last_ntp_ms == 0 || (now_ms - last_ntp_ms) >= NTP_SYNC_INTERVAL_MS;
                let retry_ok = last_ntp_attempt_ms == 0
                    || (now_ms - last_ntp_attempt_ms) >= NTP_RETRY_INTERVAL_MS;
                if ns.ip_assigned && ntp_due && retry_ok {
                    last_ntp_attempt_ms = now_ms;
                    info!("[NTP] Starting sync (ip_assigned={}, ntp_due={}, retry_ok={})",
                        ns.ip_assigned, ntp_due, retry_ok);
                    match ns.sntp_sync(now_ms) {
                        Ok(jst) => {
                            NTP_EPOCH_SECS.store(jst, Ordering::Relaxed);
                            NTP_SYNC_MS.store(now_ms as u32, Ordering::Relaxed);
                            last_ntp_ms = now_ms;
                            let (y, mo, d, h, m, s, _) = time_utils::epoch_to_parts(jst);
                            info!("[NTP] Synced → JST {:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, m, s);
                        }
                        Err(e) => warn!("[NTP] Sync failed: {:?}", e),
                    }
                }
            }

            // ── Photo drawing (direct to display, bypasses Slint renderer) ───
            if photo_needs_draw && cur_mode == MODE_PHOTO {
                if let Some(ref mut sd_reader) = sd {
                    info!("Drawing photo {}", photo_idx);
                    match sd_reader.draw_photo(photo_idx, &mut buf_provider.display) {
                        Ok(()) => {
                            photo_on_screen = true;
                            info!("Photo drawn OK");
                        }
                        Err(e) => warn!("Photo draw error: {:?}", e),
                    }
                }
                photo_needs_draw = false;
            }

            // ── Slint rendering (skipped when photo is on screen) ────────────
            if !photo_on_screen {
                if let Some(window) = self.window.borrow().clone() {
                    let mut slint_drew = false;
                    window.draw_if_needed(|renderer| {
                        slint_drew = true;
                        renderer.render_by_line(&mut buf_provider);
                    });

                    // ── Power overlay image (drawn on top of Slint UI) ────────
                    // Draw after every Slint render so the overlay is not
                    // overwritten by the background redraw.
                    if slint_drew && cur_mode == MODE_POWER {
                        if let Some(ref mut sd_reader) = sd {
                            let w = POWER_WATTS.load(Ordering::Relaxed);
                            let lvl = if w < 300 {
                                sdcard::PowerLevel::Low
                            } else if w < 1200 {
                                sdcard::PowerLevel::Mid
                            } else {
                                sdcard::PowerLevel::High
                            };
                            let lvl_u8 = lvl as u8;
                            // Only reload image when level changes to avoid
                            // SD reads every second when level is stable.
                            if power_img_level != Some(lvl_u8) {
                                match sd_reader.draw_power_image(lvl, &mut buf_provider.display) {
                                    Ok(()) => { power_img_level = Some(lvl_u8); }
                                    Err(sdcard::SdError::NoImage) => {}
                                    Err(e) => warn!("Power image: {:?}", e),
                                }
                            }
                        }
                    }

                    if window.has_active_animations() {
                        continue;
                    }
                }
            }

            {
                use embedded_hal::delay::DelayNs;
                esp_hal::delay::Delay::new().delay_ms(1u32);
            }
        }
    }
}

// ── Slint platform initializer ───────────────────────────────────────────────

pub fn init() {
    let peripherals =
        esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz));
    init_logger_from_env();
    info!("=== PowerChecker Client starting ===");
    info!("  SSID : {}", WIFI_SSID);
    info!("  URL  : {}", POWER_CHECKER_URL);
    info!("  NTP  : {}", NTP_HOST_IP);
    info!("  Heap : {} KB", HEAP_SIZE / 1024);

    esp_alloc::heap_allocator!(size: HEAP_SIZE);

    slint::platform::set_platform(Box::new(EspBackend {
        peripherals: RefCell::new(Some(peripherals)),
        window: RefCell::new(None),
    }))
    .expect("backend already initialized");
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[esp_hal::main]
fn main() -> ! {
    init();

    let app = AppWindow::new().unwrap();

    app.on_force_refresh(|| {
        FORCE_REFRESH.store(true, Ordering::Relaxed);
    });
    app.on_photo_next(|| {
        PHOTO_NEXT.store(true, Ordering::Relaxed);
    });
    app.on_photo_prev(|| {
        PHOTO_PREV.store(true, Ordering::Relaxed);
    });
    app.on_mode_changed(|mode| {
        let val = match mode {
            DisplayMode::Power => MODE_POWER,
            DisplayMode::Clock => MODE_CLOCK,
            DisplayMode::Photo => MODE_PHOTO,
        };
        CURRENT_MODE.store(val, Ordering::Relaxed);
    });

    let app_weak = app.as_weak();
    let _timer = slint::Timer::default();
    _timer.start(
        slint::TimerMode::Repeated,
        core::time::Duration::from_millis(1000),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };

            // ── Power data ────────────────────────────────────────────────────
            if DATA_VALID.load(Ordering::Relaxed) {
                let w = POWER_WATTS.load(Ordering::Relaxed);
                let ca = POWER_CENTI_A.load(Ordering::Relaxed);
                let lvl = watts_to_level(w);
                let rt = watts_to_ratio(w);
                let pct = (rt * 100.0) as u32;
                app.set_power_watts(format!("{}", w).into());
                app.set_power_amps(format!("{}.{:02}", ca / 100, ca % 100).into());
                app.set_power_level(lvl);
                app.set_power_ratio(rt);
                app.set_power_pct(format!("{}%", pct).into());
                let (_, _, _, h, m, s, _) = current_time();
                app.set_last_update(format!("{:02}:{:02}:{:02}", h, m, s).into());
            }

            // ── WiFi status ───────────────────────────────────────────────────
            app.set_wifi_ok(WIFI_CONNECTED.load(Ordering::Relaxed));

            // ── Clock ─────────────────────────────────────────────────────────
            if NTP_EPOCH_SECS.load(Ordering::Relaxed) != 0 {
                let (y, mo, d, h, m, s, dow) = current_time();
                app.set_clock_hhmm(format!("{:02}:{:02}", h, m).into());
                app.set_clock_ss(format!("{:02}", s).into());
                app.set_clock_date(format!("{:04}/{:02}/{:02}", y, mo, d).into());
                app.set_clock_dow(DOW_STR[dow as usize].into());
            } else {
                let ms = Instant::now().duration_since_epoch().as_millis();
                let s = (ms / 1000 % 60) as u8;
                let m = (ms / 60_000 % 60) as u8;
                let h = (ms / 3_600_000) as u8;
                app.set_clock_hhmm(format!("{:02}:{:02}", h, m).into());
                app.set_clock_ss(format!("{:02}", s).into());
                app.set_clock_date("Syncing NTP...".into());
                app.set_clock_dow("".into());
            }

            // ── SD card / photo status ────────────────────────────────────────
            let sd_ok = SD_OK.load(Ordering::Relaxed);
            let file_count = SD_FILE_COUNT.load(Ordering::Relaxed);
            app.set_sd_ok(sd_ok && file_count > 0);
            if sd_ok && file_count > 0 {
                app.set_photo_label(format!("{} images", file_count).into());
            } else if sd_ok {
                app.set_photo_label("No BMP in /img".into());
            } else {
                app.set_photo_label("No SD".into());
            }

            // ── Status message ────────────────────────────────────────────────
            let msg = if !WIFI_CONNECTED.load(Ordering::Relaxed) {
                "No WiFi"
            } else if !DATA_VALID.load(Ordering::Relaxed) {
                "Fetching..."
            } else {
                "OK"
            };
            app.set_status_msg(msg.into());
        },
    );

    app.run().unwrap();
    panic!("Main event loop exited")
}
