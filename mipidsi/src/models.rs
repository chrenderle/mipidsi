//! Display models.

use crate::{
    dcs::{Dcs, SetAddressMode, AsyncDcs},
    error::InitError,
    Error, ModelOptions,
};
use display_interface::{WriteOnlyDataCommand, AsyncWriteOnlyDataCommand};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

// existing model implementations
mod gc9a01;
mod ili9341;
mod ili9342c;
mod ili934x;
mod ili9486;
mod st7735s;
mod st7789;

use embedded_hal_async::delay::DelayNs;
pub use gc9a01::*;
pub use ili9341::*;
pub use ili9342c::*;
pub use ili9486::*;
pub use st7735s::*;
pub use st7789::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand;

    /// Resets the display using a reset pin.
    fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(InitError::Pin)?;

        Ok(())
    }

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    fn write_pixels<DI, I>(&mut self, di: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;

    /// Creates default [ModelOptions] for this particular [Model].
    ///
    /// This serves as a "sane default". There can be additional variants which will be provided via
    /// helper constructors.
    fn default_options() -> ModelOptions;
}

/// Display model.
pub trait AsyncModel {
    /// The color format.
    type ColorFormat: RgbColor;

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    async fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut AsyncDcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: AsyncWriteOnlyDataCommand;

    /// Resets the display using a reset pin.
    async fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10).await;
        rst.set_high().map_err(InitError::Pin)?;

        Ok(())
    }
    
    /// documentation
    fn clear(&mut self, color: Self::ColorFormat) -> Result<(), Error>;

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    fn write_pixel(&mut self, x: u16, y: u16, colors: Self::ColorFormat) -> Result<(), Error>;

    /// Creates default [ModelOptions] for this particular [Model].
    ///
    /// This serves as a "sane default". There can be additional variants which will be provided via
    /// helper constructors.
    fn default_options() -> ModelOptions;
    
    async fn flush<DI>(&mut self, dcs: &mut AsyncDcs<DI>) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand;
}