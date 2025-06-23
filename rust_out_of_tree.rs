// SPDX-License-Identifier: GPL-2.0
//! Implementation of chardev.c from LKPG

use core::sync::atomic::{AtomicBool, Ordering};
use core::pin::Pin;

use kernel::{
    error,
    c_str,
    device::Device,
    fs::File,
    ioctl::{_IOR, _IOC_SIZE},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    prelude::*,
    str::CString,
    types::ARef,
    uaccess::{UserSlice, UserSliceWriter},
};

static ALREADY_OPEN: AtomicBool = AtomicBool::new(false);

const READ_MESSAGE: u32 = _IOR::<[u8; 256]>('|' as u32, 0x83);

module! {
    type: RustMiscDeviceModule,
    name: "rust_kcounter",
    authors: ["Brandon Saint-John"],
    description: "Rust implementation of LKPG chardev",
    license: "GPL",
}

#[pin_data]
struct RustMiscDeviceModule {
    #[pin]
    _miscdev: MiscDeviceRegistration<RustMiscDevice>,
}

impl kernel::InPlaceModule for RustMiscDeviceModule {
    fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
        pr_info!("Initialising Rust Misc Device Sample\n");

        let options = MiscDeviceOptions {
            name: c_str!("kcounter-rs"),
        };

        try_pin_init!(Self {
            _miscdev <- MiscDeviceRegistration::register(options),
        })
    }
}

#[pin_data(PinnedDrop)]
struct RustMiscDevice {
    #[pin]
    msg: CString,
    dev: ARef<Device>,
}

#[vtable]
impl MiscDevice for RustMiscDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, misc: &MiscDeviceRegistration<Self>) -> Result<Pin<KBox<Self>>> {
        static mut COUNTER: i32 = 0;
        match ALREADY_OPEN.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(false) => (),
            Ok(true) => {
                pr_info!("device is open somewhere else");
                return Err(error::code::EBUSY);
            }
            Err(_) => {
                pr_info!("Failed to change lock");
                return Err(error::code::EBUSY);
            }
        }

        #[allow(static_mut_refs)]
        let msg = unsafe {
            COUNTER += 1;
            CString::try_from_fmt(fmt!("I already told you {} times Hello world!\n", COUNTER))?
        };

        let dev = ARef::from(misc.device());
        dev_info!(dev, "Opening Rust Misc Device Sample\n");

        KBox::try_pin_init(
            try_pin_init! {
                RustMiscDevice {
                    msg: msg,
                    dev: dev,
                }
            },
            GFP_KERNEL,
        )
    }

    fn ioctl(me: Pin<&RustMiscDevice>, _file: &File, cmd: u32, arg: usize) -> Result<isize> {
        dev_info!(me.dev, "IOCTLing Rust Misc Device Sample\n");

        let size = _IOC_SIZE(cmd);
        match cmd {
            READ_MESSAGE => me.message(UserSlice::new(arg, size).writer())?,
            _ => {
                dev_err!(me.dev, "-> IOCTL not recognised: {}\n", cmd);
                return Err(ENOTTY);
            }
        };

        Ok(0)
    }
}

#[pinned_drop]
impl PinnedDrop for RustMiscDevice {
    fn drop(self: Pin<&mut Self>) {
        ALREADY_OPEN.store(false, Ordering::Relaxed);
        dev_info!(self.dev, "Exiting the Rust Misc Device Sample\n");
    }
}

impl RustMiscDevice {
    fn message(&self, mut writer: UserSliceWriter) -> Result<isize> {
        let msg = self.msg.as_bytes_with_nul();
        writer.write_slice(msg)?;
        Ok(0)
    }
}
