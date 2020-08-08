#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use log::*;
use uefi::prelude::*;

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
    info!("Set output mode to: {:?}", mode);
    info!("UEFI {:?}", rev);

    loop {}
}
