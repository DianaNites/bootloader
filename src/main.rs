#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use embedded_graphics::{
    drawable::Pixel,
    egcircle,
    egtext,
    fonts::Font24x32,
    image::Image,
    pixelcolor::{RgbColor, *},
    prelude::*,
    primitive_style,
    text_style,
};
use log::*;
use tinybmp::Bmp;
use uefi::{prelude::*, proto::console::gop::*};
use uefi_graphics::UefiDisplay;

// static IMAGE: &[u8] = include_bytes!("../scratch/EFI/icons/Trans-Rust.bmp");
static IMAGE: &[u8] = include_bytes!("../scratch/EFI/icons/rust-pride.bmp");

// static IMAGE_TGA: &[u8] =
// include_bytes!("../scratch/EFI/icons/Trans-Rust.tga");
// static IMAGE_TGA: &[u8] =
// include_bytes!("../scratch/EFI/icons/rust-pride.tga");

#[entry]
fn efi_main(_img: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to init");
    let rev = st.uefi_revision();
    let stdout = st.stdout();
    trace!("Started!");
    info!("UEFI {:?}", rev);
    stdout.reset(false).unwrap_success();
    trace!("Cleared console!");

    let mode = stdout.modes().last().unwrap().log();
    info!("Setting output mode to: {:?}", mode);
    stdout.set_mode(mode).log_warning().unwrap();
    info!("Text output set mode to: {:?}", mode);
    info!("UEFI {:?}", rev);
    info!(
        "Firmware {}: {:?}",
        st.firmware_vendor(),
        st.firmware_revision()
    );
    st.boot_services()
        .set_watchdog_timer(0, u64::max_value(), None)
        .expect_success("Couldn't disable watchdog");
    info!("Attempting graphics");
    let graphics = unsafe {
        &mut *st
            .boot_services()
            .locate_protocol::<GraphicsOutput>()
            .unwrap_success()
            .get()
    };
    let mode = graphics.current_mode_info();
    info!("Current Mode: {:?}", mode);
    info!("Attempting to switch to native resolution");
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
        graphics.set_mode(&mode).unwrap().log();
    }
    let mode = graphics.current_mode_info();
    info!("New current Mode: {:?}", mode);

    let (x, y) = mode.resolution();
    let x = x / 2;
    let y = y / 2;
    let _c = egcircle!(
        center = (x as _, y as _),
        radius = (x / 2) as _,
        // style = primitive_style!(fill_color = Rgb888::new(34, 139, 34))
        style = primitive_style!(fill_color = Rgb565::BLUE)
    );
    let text = "RUST HARDWARE UEFI SAYS TRANS RIGHTS";
    let x = x - (text.len() * 12);
    let y = y - (32 / 2);
    let t = egtext!(
        text = text,
        top_left = (x as _, y as _),
        // style = text_style!(font = Font24x32, text_color = Bgr888::new(2, 136, 255))
        style = text_style!(font = Font24x32, text_color = Rgb888::new(139, 0, 139))
    );
    let bmp = Bmp::from_slice(IMAGE).expect("Failed to parse BMP image");
    let image: Image<Bmp, Rgb565> = Image::new(&bmp, Point::zero());

    // let mut display = UefiDisplay::new(unsafe { &mut *graphics.get() });
    let mut display = UefiDisplay::new(mode, graphics.frame_buffer());
    // c.draw(&mut display).unwrap();
    // image.draw(&mut display).unwrap();
    image
        .into_iter()
        .map(|Pixel(p, c)| Pixel(p, Bgr565::from(c)))
        .draw(&mut display)
        .unwrap();
    t.draw(&mut display).unwrap();

    loop {
        st.boot_services().stall(10000);
    }
}
