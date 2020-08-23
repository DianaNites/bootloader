#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use rlibc as _;
use uefi::prelude::*;

#[entry]
fn efi_main(_img: Handle, st: SystemTable<Boot>) -> Status {
    fn res() -> (usize, usize) {
        (1280u32 as usize, 768u32 as usize)
    }
    let (display_width, display_height) = res();
    let (_display_width, _display_height) =
        ((display_width / 2) as f32, (display_height / 2) as f32);
    loop {
        st.boot_services().stall(1000)
    }
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    let mut x = 0;
    loop {
        x += 1;
        if x == !0 {
            break;
        }
    }
    panic!("{}", x);
}
