mod base;
mod camera;
mod real_sense;
mod gpu;
mod i2c;
mod gpio;
mod isp;
mod codec;
mod rendering;
mod other;
mod generic_device;

use crate::peripherals::camera::*;
use crate::peripherals::codec::*;
use crate::peripherals::generic_device::*;
use crate::peripherals::gpio::*;
use crate::peripherals::gpu::*;
use crate::peripherals::i2c::*;
use crate::peripherals::isp::*;
use crate::peripherals::other::*;
use crate::peripherals::rendering::*;
use crate::peripherals::real_sense::*;


#[derive(Debug, Clone)]
pub enum Peripheral {
    GPU(GpuPeripheral),
    I2C(I2cPeripheral),
    GPIO(GpioPeripheral),
    Camera(CameraPeripheral),
    RealSense(RealSenseCameraPeripheral),
    ISP(IspPeripheral),
    Codec(CodecPeripheral),
    Rendering(RenderingPeripheral),
    Speaker(OtherPeripheral),
    Keyboard(OtherPeripheral),
    Mouse(OtherPeripheral),
    GenericDevice(GenericDevicePeripheral),
    Other(OtherPeripheral),
}
