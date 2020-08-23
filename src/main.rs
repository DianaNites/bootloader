#![allow(unused_variables)]
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
    // This will lead to a crash? thing? qemu will infinitely reset.
    let (display_width, display_height) = res();

    // This will not.
    // let (display_width, display_height) = (1280u32 as usize, 768u32 as usize);

    // The actual crash will only happen if this is here.
    let (display_width, display_height) = ((display_width / 2) as f64, (display_height / 2) as f64);

    // Loop forever and also do something.
    loop {
        st.boot_services().stall(1000)
    }
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    let mut x = 0;
    // Loop forever and also do something.
    loop {
        x += 1;
        if x == !0 {
            break;
        }
    }
    panic!("{}", x);
}
