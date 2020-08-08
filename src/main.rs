#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use log::info;
use uefi::prelude::*;

#[entry]
fn efi_main(_img: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to init");

    let rev = st.uefi_revision();
    info!("UEFI {:?}", rev);

    info!("{:?}", st.stdout().current_mode());

    Status::SUCCESS
}
