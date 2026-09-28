//! Minimal bus boundary that keeps individual device ports independently testable.

pub trait Bus {
    fn open_bus(&self) -> u8;
    fn read(&mut self, address: u32) -> u8;
    fn write(&mut self, address: u32, value: u8);
    fn read_b_bus(&mut self, address: u8) -> u8;
    fn write_b_bus(&mut self, address: u8, value: u8);
}

/// Memory-mapped APU surface consumed by the SPC700 instruction core.
pub trait ApuBus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}
