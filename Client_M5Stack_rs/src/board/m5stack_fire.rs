use core::cell::RefCell;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode as SpiMode,
    },
    time::Rate,
};
use log::info;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation};
use static_cell::StaticCell;

pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;
// 96 KB heap for internal SRAM.
// WiFi stack alone needs ~72 KB; BMP decode is row-by-row (stack only),
// so no large heap allocations are needed for image rendering.
// Total DRAM budget: ~80 KB WiFi static + 96 KB heap + ~10 KB other ≈ 186 KB
// — well within the ~328 KB available on ESP32.
pub const HEAP_SIZE: usize = 96 * 1024;

pub type SharedSpiDev =
    RefCellDevice<'static, Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>;

pub type BoardDisplay = mipidsi::Display<
    mipidsi::interface::SpiInterface<'static, SharedSpiDev, Output<'static>>,
    mipidsi::models::ILI9342CRgb565,
    Output<'static>,
>;

pub struct BoardPeripherals {
    pub btn_a: Input<'static>,
    pub btn_b: Input<'static>,
    pub btn_c: Input<'static>,
    pub display: BoardDisplay,
    pub sd_spi: SharedSpiDev,
    pub wifi_timer_peripheral: esp_hal::peripherals::TIMG1<'static>,
    pub rng_peripheral: esp_hal::peripherals::RNG<'static>,
    pub wifi_peripheral: esp_hal::peripherals::WIFI<'static>,
}

static SPI_BUS: StaticCell<RefCell<Spi<'static, esp_hal::Blocking>>> = StaticCell::new();

#[allow(static_mut_refs)]
static mut SPI_BUF: [u8; 512] = [0u8; 512];

pub fn init_hardware(peripherals: Peripherals) -> BoardPeripherals {
    let mut delay = Delay::new();

    let btn_a = Input::new(peripherals.GPIO39, InputConfig::default());
    let btn_b = Input::new(peripherals.GPIO38, InputConfig::default());
    let btn_c = Input::new(peripherals.GPIO37, InputConfig::default());

    let spi = Spi::<esp_hal::Blocking>::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(20))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23)
    .with_miso(peripherals.GPIO19);

    let spi_bus: &'static RefCell<Spi<'static, esp_hal::Blocking>> =
        SPI_BUS.init(RefCell::new(spi));

    let display_cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let sd_cs = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO33, Level::High, OutputConfig::default());
    let mut _backlight = Output::new(peripherals.GPIO32, Level::High, OutputConfig::default());

    let display_spi = RefCellDevice::new(spi_bus, display_cs, Delay::new()).unwrap();
    let sd_spi = RefCellDevice::new(spi_bus, sd_cs, Delay::new()).unwrap();

    #[allow(static_mut_refs)]
    let di = mipidsi::interface::SpiInterface::new(display_spi, dc, unsafe { &mut SPI_BUF });

    delay.delay_ms(10u32);
    let mut display = mipidsi::Builder::new(mipidsi::models::ILI9342CRgb565, di)
        .reset_pin(reset)
        .display_size(320, 240)
        .orientation(Orientation::default())
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
    display.clear(Rgb565::BLACK).unwrap();
    info!("Display initialized (M5Stack Fire ILI9342C)");

    BoardPeripherals {
        btn_a,
        btn_b,
        btn_c,
        display,
        sd_spi,
        wifi_timer_peripheral: peripherals.TIMG1,
        rng_peripheral: peripherals.RNG,
        wifi_peripheral: peripherals.WIFI,
    }
}

pub struct DrawBuffer<'a, D> {
    pub display: D,
    pub buffer: &'a mut [slint::platform::software_renderer::Rgb565Pixel],
}

impl<
    DI: mipidsi::interface::Interface<Word = u8>,
    RST: OutputPin<Error = core::convert::Infallible>,
> slint::platform::software_renderer::LineBufferProvider
    for &mut DrawBuffer<'_, mipidsi::Display<DI, mipidsi::models::ILI9342CRgb565, RST>>
{
    type TargetPixel = slint::platform::software_renderer::Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [slint::platform::software_renderer::Rgb565Pixel]),
    ) {
        let buffer = &mut self.buffer[range.clone()];
        render_fn(buffer);
        self.display
            .set_pixels(
                range.start as u16,
                line as u16,
                (range.end - 1) as u16,
                line as u16,
                buffer.iter().map(|x| {
                    embedded_graphics_core::pixelcolor::raw::RawU16::new(x.0).into()
                }),
            )
            .unwrap();
    }
}
