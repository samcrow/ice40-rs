use hal::{spidev::SpidevOptions, sysfs_gpio::Direction};
use std::{fs, path::PathBuf, thread::sleep, time::Duration};
use structopt::StructOpt;

use linux_embedded_hal as hal;

#[derive(Debug, StructOpt)]
#[structopt(name = concat!(env!("CARGO_PKG_NAME"), "/linux"), about = "Linux demo")]
struct Opt {
    #[structopt(
        long,
        parse(from_os_str),
        default_value = "/dev/spidev0.0",
        help = "SPI bus"
    )]
    spi: PathBuf,

    #[structopt(long, default_value = "8", help = "SS pin")]
    ss: u64,

    #[structopt(long, default_value = "25", help = "CRESET pin")]
    creset: u64,

    #[structopt(long, default_value = "24", help = "CDONE pin")]
    cdone: u64,

    /// Set speed
    #[structopt(short, long, default_value = "3000000", help = "Bus frequency")]
    frequency: u32,

    /// Input file
    #[structopt(parse(from_os_str), help = "Binary file")]
    binary: PathBuf,
}

struct DummyDelay;
impl embedded_hal::delay::DelayNs for DummyDelay {
    fn delay_ns(&mut self, ns: u32) {
        sleep(Duration::from_nanos(ns.into()))
    }
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let spiopt = SpidevOptions::new()
        .max_speed_hz(opt.frequency)
        .lsb_first(false)
        .build();

    let bitstream = fs::read(opt.binary).expect("Failed to read binary file");
    log::info!("Read binary file, size = {}", bitstream.len());

    let mut spi = hal::SpidevBus::open(opt.spi).expect("Failed to open SPI bus");
    spi.configure(&spiopt).expect("Failed to configure SPI bus");
    let ss = hal::SysfsPin::new(opt.ss);
    ss.export().expect("Failed to export SS pin");
    ss.set_direction(Direction::Out).unwrap();
    let done = hal::SysfsPin::new(opt.cdone);
    done.export().expect("Failed to export CDONE pin");
    done.set_direction(Direction::In).unwrap();
    let reset = hal::SysfsPin::new(opt.creset);
    reset.export().expect("Failed to export CRESET pin");
    reset.set_direction(Direction::Out).unwrap();

    log::info!("Configuring device...");
    let mut device = ice40::Device::new(spi, ss, done, reset);
    device
        .configure(&mut DummyDelay, &bitstream[..])
        .expect("Failed to configure FPGA");
    log::info!("done!");
}
