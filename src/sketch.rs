use crate::core::VirtualMcu;

pub trait Sketch {
    /// Runs once when simulation begins
    fn setup(&mut self, mcu: &mut VirtualMcu);

    /// Runs infinitely per tick
    fn run_loop(&mut self, mcu: &mut VirtualMcu);
}
