#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use core::convert::{Infallible, TryInto};
use embedded_graphics::{
    drawable::Pixel,
    egcircle,
    egtext,
    fonts::{Font24x32, Font6x8, Text},
    mock_display::MockDisplay,
    pixelcolor::{raw::*, Rgb565, RgbColor, *},
    prelude::*,
    primitive_style,
    primitives::Circle,
    style::{PrimitiveStyle, TextStyle},
    text_style,
};
use log::*;
use uefi::{prelude::*, proto::console::gop::*};

struct Dis<'a, 'b> {
    graphics: &'a mut GraphicsOutput<'b>,
}

impl<'a, 'b> Dis<'a, 'b> {
    fn new(graphics: &'a mut GraphicsOutput<'b>) -> Self {
        Self { graphics }
    }
}

impl<'a, 'b, T: Into<Bgr888> + PixelColor> DrawTarget<T> for Dis<'a, 'b> {
    type Error = Infallible;

    fn draw_pixel(&mut self, item: Pixel<T>) -> Result<(), Self::Error> {
        let Pixel(point, color) = item;
        let color = color.into();

        let mode = self.graphics.current_mode_info();
        let (max_x, max_y) = mode.resolution();
        let (x, y) = (point.x as usize, point.y as usize);
        if x < max_x && y < max_y {
            let index = y * mode.stride() + x;

            let mut fb = self.graphics.frame_buffer();
            unsafe {
                // TODO: Dynamic, support other things.
                // count_ones on mask?
                fb.write_value(index * 4, color)
            }
        } else {
            warn!("Tried to draw out of bounds");
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let (width, height) = self.graphics.current_mode_info().resolution();
        Size::new(width as u32, height as u32)
    }

    fn clear(&mut self, color: T) -> Result<(), Self::Error> {
        let (width, height) = self.graphics.current_mode_info().resolution();
        let color: Bgr888 = color.into();
        self.graphics
            .blt(BltOp::VideoFill {
                color: BltPixel::new(color.r(), color.g(), color.b()),
                dest: (0, 0),
                dims: (width - 1, height - 1),
            })
            .unwrap_success();
        Ok(())
    }
}

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
    let graphics = st
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .unwrap_success();
    let mode;
    unsafe {
        let g = &*graphics.get();
        mode = g.current_mode_info();
    }
    info!("Current Mode: {:?}", mode);
    // unsafe {
    //     let g = &*graphics.get();
    //     for mode in g.modes() {
    //         let mode = mode.unwrap();
    //         info!("{:?}", mode.info());
    //     }
    // }

    let (x, y) = mode.resolution();
    let x = x / 2;
    let y = y / 2;
    let c = egcircle!(
        center = (x as _, y as _),
        radius = (x / 2) as _,
        style = primitive_style!(fill_color = Bgr888::new(38, 0, 27))
    );
    let text = "FUCK GRAPHICS I HATE YOU";
    let x = x - (text.len() * 12);
    let y = y - (32 / 2);
    let t = egtext!(
        text = text,
        top_left = (x as _, y as _),
        // style = text_style!(font = Font24x32, text_color = Bgr888::new(2, 136, 255))
        style = text_style!(font = Font24x32, text_color = Bgr888::new(70, 130, 180))
    );

    let mut display = Dis::new(unsafe { &mut *graphics.get() });
    c.draw(&mut display).unwrap();
    t.draw(&mut display).unwrap();

    loop {
        st.boot_services().stall(10000);
    }
}
