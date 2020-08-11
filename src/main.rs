#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use alloc::format;
use embedded_graphics::{
    fonts::Text,
    image::Image,
    pixelcolor::{RgbColor, *},
    style::TextStyleBuilder,
    DrawTarget,
};
use embedded_layout::prelude::*;
use log::*;
use tinybmp::Bmp;
use uefi::{
    prelude::*,
    proto::{
        console::{gop::*, text::Output},
        media::fs::SimpleFileSystem,
    },
};
use uefi_graphics::{UefiDisplay, UefiDisplayNotGeneric};
extern crate alloc;

static _TRANS_RUST_BMP: &[u8] = include_bytes!("../scratch/EFI/icons/Trans-Rust.bmp");
static RUST_PRIDE_BMP: &[u8] = include_bytes!("../scratch/EFI/icons/rust-pride.bmp");

static _TRANS_RUST_TGA: &[u8] = include_bytes!("../scratch/EFI/icons/Trans-Rust.tga");
static _RUST_PRIDE_TGA: &[u8] = include_bytes!("../scratch/EFI/icons/rust-pride.tga");

/// Returns `Some(GraphicsOutput)` if graphical output is supported
fn graphics_supported(st: &SystemTable<Boot>) -> Option<&mut GraphicsOutput> {
    unsafe {
        st.boot_services()
            .locate_protocol::<GraphicsOutput>()
            .map(|c| c.log())
            .ok()
            .map(|g| &mut *g.get())
    }
}

/// Setup the screen mode
fn setup_screen(graphics: &mut GraphicsOutput) -> Status {
    info!("Attempting to switch to native resolution");
    // let best_mode = graphics.modes().last().ok_or(Status::NOT_FOUND)?.log();
    // graphics.set_mode(&best_mode)?.log();
    let mut new_mode = None;
    for mode in graphics.modes() {
        let mode = mode.unwrap();
        // NOTE: QEMU Hack.
        if mode.info().resolution() == (1280, 768) {
            new_mode = Some(mode);
            break;
        }
    }
    if let Some(mode) = new_mode {
        info!("New current Mode: {:?}", mode.info());
        graphics.set_mode(&mode)?.log();
    }
    Status::SUCCESS
}

/// Graphical display
fn graphical_ui(_st: &SystemTable<Boot>, graphics: &mut GraphicsOutput) {
    let mode = graphics.current_mode_info();
    info!("Current Mode: {:?}", mode);

    let display = &mut UefiDisplay::new(mode, graphics.frame_buffer());
    display.clear(Bgr888::BLACK).unwrap();

    let text_style = TextStyleBuilder::new(embedded_graphics::fonts::Font8x16)
        .text_color(Rgb888::BLUE)
        .build();

    let bmp = Bmp::from_slice(RUST_PRIDE_BMP).expect("Failed to parse BMP image");
    let rust_pride: Image<Bmp, Rgb565> = Image::new(&bmp, Point::zero());
    let t = Text::new("rust-pride.bmp, Generic, Image<Bmp, Rgb565>", Point::zero())
        .into_styled(text_style)
        .align_to(&rust_pride, horizontal::NoAlignment, vertical::TopToBottom);

    let rust_pride_bgr = Image::<Bmp, Bgr565>::new(&bmp, Point::zero()).align_to(
        &t,
        horizontal::NoAlignment,
        vertical::TopToBottom,
    );
    let t_bgr = Text::new("rust-pride.bmp, Generic, Image<Bmp, Bgr565>", Point::zero())
        .into_styled(text_style)
        .align_to(
            &rust_pride_bgr,
            horizontal::NoAlignment,
            vertical::TopToBottom,
        );

    rust_pride.draw(display).unwrap();
    rust_pride_bgr.draw(display).unwrap();
    t.draw(display).unwrap();
    t_bgr.draw(display).unwrap();

    let display = &mut UefiDisplayNotGeneric::new(mode, graphics.frame_buffer());

    let text_style = TextStyleBuilder::new(embedded_graphics::fonts::Font8x16)
        .text_color(Bgr888::BLUE)
        .build();

    let bmp = Bmp::from_slice(RUST_PRIDE_BMP).expect("Failed to parse BMP image");
    let rust_pride: Image<Bmp, Rgb565> = Image::new(&bmp, Point::zero()).align_to(
        &t_bgr,
        horizontal::NoAlignment,
        vertical::TopToBottom,
    );
    let t = Text::new(
        "rust-pride.bmp, Manually convert pixels, Image<Bmp, Rgb565>",
        Point::zero(),
    )
    .into_styled(text_style)
    .align_to(&rust_pride, horizontal::NoAlignment, vertical::TopToBottom);

    let rust_pride_bgr = Image::<Bmp, Bgr565>::new(&bmp, Point::zero()).align_to(
        &t,
        horizontal::NoAlignment,
        vertical::TopToBottom,
    );
    let t_bgr = Text::new(
        "rust-pride.bmp, Manually convert pixels, Image<Bmp, Bgr565>",
        Point::zero(),
    )
    .into_styled(text_style)
    .align_to(
        &rust_pride_bgr,
        horizontal::NoAlignment,
        vertical::TopToBottom,
    );

    rust_pride
        .into_iter()
        .map(|Pixel(p, c)| Pixel(p, Bgr888::from(c)))
        // .map(|Pixel(p, c)| Pixel(p, Bgr888::new(c.r(), c.g(), c.b())))
        .draw(display)
        .unwrap();
    rust_pride_bgr
        .into_iter()
        .map(|Pixel(p, c)| Pixel(p, Bgr888::from(c)))
        // .map(|Pixel(p, c)| Pixel(p, Bgr888::new(c.r(), c.g(), c.b())))
        .draw(display)
        .unwrap();
    t.draw(display).unwrap();
    t_bgr.draw(display).unwrap();

    let text = format!("Current Mode Info: {:#?}", mode);
    let info = Text::new(&text, Point::zero())
        .into_styled(text_style)
        .align_to(&t_bgr, horizontal::NoAlignment, vertical::TopToBottom);

    info.draw(display).unwrap();
}

/// Check whether the system supports what we require.
///
/// Currently we require at least UEFI 2.3.1, and the following protocols:
///
/// - Text Output
/// - Block IO
/// - Disk IO
/// - Simple Filesystem
///
/// And optionally support:
///
/// - Graphics output
/// - Simple Pointer
/// - PXE Base Code
/// - Network Interface Identifier
/// - Simple Network
/// - Managed Network
/// - HTTP Service Binding
/// - HTTP
/// - HTTP Utilities
/// - TLS Service Binding
/// - TLS
/// - DNS4 Service Binding
/// - DNS4
/// - EAP
/// - EAP Configuration
/// - EAP Management 2
/// - Supplicant
fn check_support(st: &SystemTable<Boot>) -> Status {
    let boot = st.boot_services();
    let _text = boot
        .locate_protocol::<Output>()
        .map_err(|_| Status::UNSUPPORTED)?;
    let _fs = boot
        .locate_protocol::<SimpleFileSystem>()
        .map_err(|_| Status::UNSUPPORTED)?;
    let rev = st.uefi_revision();
    info!(
        "Using UEFI: {:?} with Firmware: {}: {:?}",
        rev,
        st.firmware_vendor(),
        st.firmware_revision()
    );
    if rev.major() < 2 || (rev.major() == 2 && rev.minor() < 31) {
        error!("Unsupported UEFI version");
        return Status::UNSUPPORTED;
    }
    Status::SUCCESS
}

fn setup_term(stdout: &mut Output) -> Status {
    let best_mode = stdout.modes().last().unwrap().unwrap();
    stdout.set_mode(best_mode)?.log();
    warn!("If you can see this, the UEFI console didn't properly reset.");
    stdout.reset(false)?.log();
    trace!("Cleared console!");
    info!("Current terminal mode: {:?}.", best_mode);
    Status::SUCCESS
}

#[entry]
fn efi_main(_img: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to init");
    st.boot_services()
        .set_watchdog_timer(0, u64::max_value(), None)?
        .log();
    setup_term(st.stdout())?.log();
    check_support(&st)?.log();

    info!("Initializing Graphics");
    if let Some(graphics) = graphics_supported(&st) {
        setup_screen(graphics)?.log();
        graphical_ui(&st, graphics);
    } else {
        // TODO: Terminal
    }

    loop {
        st.boot_services().stall(10000)
    }
}
