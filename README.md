# UEFI problem reproducer

Instructions

Run

```shell
cargo build && mkdir -p ./scratch/EFI/Boot && cp ./target/x86_64-unknown-uefi/debug/bootloader.efi ./scratch/EFI/Boot/BootX64.efi
```

Then

```shell
qemu-system-x86_64 -nodefaults -machine q35 -smp 3 -m 128M --enable-kvm -drive if=pflash,format=raw,file=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd,readonly=on -drive if=pflash,format=raw,file=/usr/share/edk2-ovmf/x64/OVMF_VARS.fd,readonly=on -drive format=raw,file=fat:rw:scratch -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -vga std
```

Note the above qemu command expects Arch Linux with qemu and edk2-ovmf installed.
You may need to change the paths.

## System

* Arch Linux
* QEMU emulator version 5.1.0
* rustc 1.47.0-nightly (663d2f5cd 2020-08-22)
