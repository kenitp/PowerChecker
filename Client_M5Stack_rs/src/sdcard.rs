// SD card image reader.
//
// Directory layout on the SD card (matches original C++ project):
//   /IMG/POWER/LOW/   *.bmp  → overlay at x=185, y=55 (140×184 px)
//   /IMG/POWER/MID/   *.bmp
//   /IMG/POWER/HIGH/  *.bmp
//   /IMG/             *.bmp  → photo slideshow (full-screen)
//
// BMP is decoded row-by-row directly to the display — no heap buffer
// is allocated for pixel data, making this RAM-safe even without PSRAM.
//
// Note: The original C++ project stored JPEG/PNG images.  On bare-metal
// ESP32 (no PSRAM), decoding a 320×240 JPEG requires ~230 KB heap which
// is unavailable alongside the WiFi stack (~80 KB static + ~72 KB heap).
// BMP at 16/24/32 bpp is decoded one row at a time and avoids this limit.
// Convert images to BMP on the host before copying to the SD card, e.g.:
//   convert input.jpg -type TrueColor BMP3:output.bmp
//
// Power image draw position matches draw_power.cpp: x=185, y=55, 140×184 px.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::delay::Delay;
use log::{info, warn};

use crate::board::SharedSpiDev;

// Power image draw position (matches C++ draw_power.cpp)
pub const POWER_IMG_X: u16 = 185;
pub const POWER_IMG_Y: u16 = 55;
pub const POWER_IMG_MAX_W: u32 = 140;
pub const POWER_IMG_MAX_H: u32 = 184;

pub type SdCardDev = SdCard<SharedSpiDev, Delay>;
type VolMgr = VolumeManager<SdCardDev, DummyTimeSource>;

struct DummyTimeSource;
impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2025, 1, 1, 0, 0, 0).unwrap()
    }
}

/// Power level index (matches config.cpp ordering: low=0, mid=1, high=2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerLevel {
    Low = 0,
    Mid = 1,
    High = 2,
}

#[derive(Debug)]
pub enum SdError {
    Init,
    NoVolume,
    NoDir,
    FileOpen,
    Read,
    BadBmp,
    NoImage,
}

// ── BMP helpers ───────────────────────────────────────────────────────────────

struct BmpInfo {
    width: u32,
    height: u32,
    bits_per_pixel: u16,
    data_offset: u32,
    row_stride: u32,
    top_down: bool,
}

fn parse_bmp_header(hdr: &[u8; 54]) -> Result<BmpInfo, SdError> {
    if hdr[0] != b'B' || hdr[1] != b'M' {
        return Err(SdError::BadBmp);
    }
    let data_offset = u32::from_le_bytes([hdr[10], hdr[11], hdr[12], hdr[13]]);
    let width = i32::from_le_bytes([hdr[18], hdr[19], hdr[20], hdr[21]]);
    let height = i32::from_le_bytes([hdr[22], hdr[23], hdr[24], hdr[25]]);
    let bits_per_pixel = u16::from_le_bytes([hdr[28], hdr[29]]);

    if !matches!(bits_per_pixel, 16 | 24 | 32) {
        warn!("Unsupported BMP bpp: {}", bits_per_pixel);
        return Err(SdError::BadBmp);
    }

    let top_down = height < 0;
    let abs_h = height.unsigned_abs();
    let abs_w = width.unsigned_abs();
    let bytes_pp = (bits_per_pixel as u32 + 7) / 8;
    let row_stride = ((abs_w * bytes_pp + 3) / 4) * 4;

    Ok(BmpInfo {
        width: abs_w,
        height: abs_h,
        bits_per_pixel,
        data_offset,
        row_stride,
        top_down,
    })
}

fn row_to_rgb565(raw: &[u8], bpp: u16, pixels: usize, out: &mut [u16]) {
    match bpp {
        16 => {
            for i in 0..pixels.min(raw.len() / 2) {
                out[i] = u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]]);
            }
        }
        24 => {
            for i in 0..pixels.min(raw.len() / 3) {
                let b = raw[i * 3] as u16;
                let g = raw[i * 3 + 1] as u16;
                let r = raw[i * 3 + 2] as u16;
                out[i] = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3);
            }
        }
        32 => {
            for i in 0..pixels.min(raw.len() / 4) {
                let b = raw[i * 4] as u16;
                let g = raw[i * 4 + 1] as u16;
                let r = raw[i * 4 + 2] as u16;
                out[i] = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3);
            }
        }
        _ => {}
    }
}

/// Draw a BMP file to the display at `(x_off, y_off)`, clipped to `max_w × max_h`.
/// Decodes one row at a time — no heap allocation for pixel data.
fn draw_bmp_at<'a, D, T, const MD: usize, const MF: usize, const MV: usize, DI, RST>(
    file: &mut embedded_sdmmc::File<'a, D, T, MD, MF, MV>,
    display: &mut mipidsi::Display<DI, mipidsi::models::ILI9342CRgb565, RST>,
    x_off: u16,
    y_off: u16,
    max_w: u32,
    max_h: u32,
) -> Result<(), SdError>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    DI: mipidsi::interface::Interface<Word = u8>,
    RST: embedded_hal::digital::OutputPin<Error = core::convert::Infallible>,
{
    let mut hdr = [0u8; 54];
    let n = file.read(&mut hdr).map_err(|_| SdError::Read)?;
    if n < 54 {
        return Err(SdError::BadBmp);
    }
    let info = parse_bmp_header(&hdr)?;
    info!("BMP: {}×{} {}bpp", info.width, info.height, info.bits_per_pixel);

    let draw_w = info.width.min(max_w);
    let draw_h = info.height.min(max_h);
    let bpp_bytes = (info.bits_per_pixel as usize + 7) / 8;

    // Row buffers on the stack — no heap allocation.
    let mut raw_buf = [0u8; 960]; // fits a 320-wide 24bpp row (320×3 = 960)
    let mut px_buf = [0u16; 320];

    for y in 0..draw_h {
        let file_row = if info.top_down {
            y
        } else {
            info.height - 1 - y
        };
        let offset = info.data_offset + file_row * info.row_stride;
        file.seek_from_start(offset).map_err(|_| SdError::Read)?;

        let to_read = (draw_w as usize * bpp_bytes).min(raw_buf.len());
        let n = file.read(&mut raw_buf[..to_read]).map_err(|_| SdError::Read)?;

        row_to_rgb565(&raw_buf[..n], info.bits_per_pixel, draw_w as usize, &mut px_buf);

        let iter = px_buf[..draw_w as usize].iter().map(|&raw| {
            embedded_graphics_core::pixelcolor::raw::RawU16::new(raw).into()
        });
        display
            .set_pixels(
                x_off,
                y_off + y as u16,
                x_off + draw_w as u16 - 1,
                y_off + y as u16,
                iter,
            )
            .ok();
    }
    Ok(())
}

// ── SdReader ─────────────────────────────────────────────────────────────────

pub struct SdReader {
    mgr: VolMgr,
    /// BMP filenames (8.3 format) under /IMG/POWER/{LOW,MID,HIGH}
    pub power_files: [Vec<String>; 3],
    /// BMP filenames (8.3 format) under /IMG/  (slideshow)
    pub photo_files: Vec<String>,
    /// Cycling counter per power level
    power_cycle: [u32; 3],
}

impl SdReader {
    pub fn new(spi: SharedSpiDev) -> Result<Self, SdError> {
        let sdcard = SdCard::new(spi, Delay::new());
        match sdcard.num_bytes() {
            Ok(bytes) => info!("SD: {} MB", bytes / (1024 * 1024)),
            Err(e) => {
                warn!("SD init failed: {:?}", e);
                return Err(SdError::Init);
            }
        }
        let mgr = VolumeManager::new(sdcard, DummyTimeSource);
        Ok(SdReader {
            mgr,
            power_files: [Vec::new(), Vec::new(), Vec::new()],
            photo_files: Vec::new(),
            power_cycle: [0; 3],
        })
    }

    /// Scan all image directories.  Returns total image count.
    pub fn scan_all(&mut self) -> usize {
        let vol = match self.mgr.open_volume(VolumeIdx(0)) {
            Ok(v) => v,
            Err(e) => { warn!("open_volume: {:?}", e); return 0; }
        };
        let root = match vol.open_root_dir() {
            Ok(d) => d,
            Err(e) => { warn!("open_root_dir: {:?}", e); return 0; }
        };
        let img_dir = match root.open_dir("IMG") {
            Ok(d) => d,
            Err(_) => { warn!("No /IMG directory"); return 0; }
        };

        // /IMG/  — photo slideshow
        let mut photos: Vec<String> = Vec::new();
        let _ = img_dir.iterate_dir(|e| {
            if !e.attributes.is_directory() {
                let name = e.name.to_string();
                if is_bmp(&name) {
                    photos.push(name);
                }
            }
        });
        info!("Slideshow BMPs in /IMG: {}", photos.len());
        self.photo_files = photos;

        // /IMG/POWER/{LOW,MID,HIGH}
        match img_dir.open_dir("POWER") {
            Ok(power_dir) => {
                for (i, sub) in ["LOW", "MID", "HIGH"].iter().enumerate() {
                    match power_dir.open_dir(*sub) {
                        Ok(sub_dir) => {
                            let mut files: Vec<String> = Vec::new();
                            let _ = sub_dir.iterate_dir(|e| {
                                if !e.attributes.is_directory() {
                                    let name = e.name.to_string();
                                    if is_bmp(&name) {
                                        files.push(name);
                                    }
                                }
                            });
                            info!("Power/{}: {} BMPs", sub, files.len());
                            self.power_files[i] = files;
                        }
                        Err(_) => warn!("No /IMG/POWER/{}", sub),
                    }
                }
            }
            Err(_) => warn!("No /IMG/POWER directory"),
        }

        self.photo_files.len()
            + self.power_files[0].len()
            + self.power_files[1].len()
            + self.power_files[2].len()
    }

    pub fn photo_count(&self) -> usize {
        self.photo_files.len()
    }

    /// Draw power overlay BMP at x=185, y=55 (max 140×184 px).
    /// Cycles through available images for the given level.
    pub fn draw_power_image<DI, RST>(
        &mut self,
        level: PowerLevel,
        display: &mut mipidsi::Display<DI, mipidsi::models::ILI9342CRgb565, RST>,
    ) -> Result<(), SdError>
    where
        DI: mipidsi::interface::Interface<Word = u8>,
        RST: embedded_hal::digital::OutputPin<Error = core::convert::Infallible>,
    {
        let idx = level as usize;
        if self.power_files[idx].is_empty() {
            return Err(SdError::NoImage);
        }
        let cycle = self.power_cycle[idx] as usize % self.power_files[idx].len();
        self.power_cycle[idx] = self.power_cycle[idx].wrapping_add(1);
        let filename = self.power_files[idx][cycle].clone();

        let subdir = match level {
            PowerLevel::Low  => "LOW",
            PowerLevel::Mid  => "MID",
            PowerLevel::High => "HIGH",
        };

        let vol = self.mgr.open_volume(VolumeIdx(0)).map_err(|_| SdError::NoVolume)?;
        let root = vol.open_root_dir().map_err(|_| SdError::NoDir)?;
        let img_dir = root.open_dir("IMG").map_err(|_| SdError::NoDir)?;
        let power_dir = img_dir.open_dir("POWER").map_err(|_| SdError::NoDir)?;
        let level_dir = power_dir.open_dir(subdir).map_err(|_| SdError::NoDir)?;
        let mut file = level_dir
            .open_file_in_dir(&*filename, Mode::ReadOnly)
            .map_err(|e| { warn!("open {}: {:?}", filename, e); SdError::FileOpen })?;

        draw_bmp_at(
            &mut file,
            display,
            POWER_IMG_X,
            POWER_IMG_Y,
            POWER_IMG_MAX_W,
            POWER_IMG_MAX_H,
        )
    }

    /// Draw slideshow photo `idx` full-screen.
    pub fn draw_photo<DI, RST>(
        &mut self,
        idx: usize,
        display: &mut mipidsi::Display<DI, mipidsi::models::ILI9342CRgb565, RST>,
    ) -> Result<(), SdError>
    where
        DI: mipidsi::interface::Interface<Word = u8>,
        RST: embedded_hal::digital::OutputPin<Error = core::convert::Infallible>,
    {
        if idx >= self.photo_files.len() {
            return Err(SdError::NoImage);
        }
        let filename = self.photo_files[idx].clone();

        let vol = self.mgr.open_volume(VolumeIdx(0)).map_err(|_| SdError::NoVolume)?;
        let root = vol.open_root_dir().map_err(|_| SdError::NoDir)?;
        let img_dir = root.open_dir("IMG").map_err(|_| SdError::NoDir)?;
        let mut file = img_dir
            .open_file_in_dir(&*filename, Mode::ReadOnly)
            .map_err(|e| { warn!("open {}: {:?}", filename, e); SdError::FileOpen })?;

        draw_bmp_at(
            &mut file,
            display,
            0,
            0,
            crate::board::DISPLAY_WIDTH,
            crate::board::DISPLAY_HEIGHT,
        )
    }
}

fn is_bmp(name: &str) -> bool {
    let u = name.to_uppercase();
    u.ends_with(".BMP")
}
