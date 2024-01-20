use defmt::info;
use display_interface::{DataFormat, WriteOnlyDataCommand, AsyncWriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};
use embedded_hal_async::delay::DelayNs as AsyncDelayNs;

use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset, WriteMemoryStart, AsyncDcs,
    },
    error::InitError,
    ColorInversion, Error, ModelOptions,
};

use super::{Model, AsyncModel};

/// Module containing all ST7789 variants.
mod variants;

/// ST7789 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7789;
pub struct ST7789Framebuffer<'framebuffer> {
    framebuffer: &'framebuffer mut [u16; 240 * 135],
}

impl Model for ST7789 {
    type ColorFormat = Rgb565;

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
        DI: WriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }
        delay.delay_us(150_000);

        dcs.write_command(ExitSleepMode)?;
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        dcs.write_command(SetScrollArea::from(options))?;
        dcs.write_command(madctl)?;

        dcs.write_command(SetInvertMode(options.invert_colors))?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?;
        delay.delay_us(10_000);
        dcs.write_command(EnterNormalMode)?;
        delay.delay_us(10_000);
        dcs.write_command(SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;

        let mut iter = colors.into_iter().map(Rgb565::into_storage);

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        let mut options = ModelOptions::with_sizes((240, 320), (240, 320));
        options.set_invert_colors(ColorInversion::Normal);

        options
    }
}

impl<'framebuffer> AsyncModel for ST7789Framebuffer<'framebuffer> {
    type ColorFormat = Rgb565;

    async fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut AsyncDcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: AsyncDelayNs,
        DI: AsyncWriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }
        delay.delay_us(150_000).await;

        dcs.write_command(ExitSleepMode).await?;
        delay.delay_us(10_000).await;

        // set hw scroll area based on framebuffer size
        dcs.write_command(SetScrollArea::from(options)).await?;
        dcs.write_command(madctl).await?;

        dcs.write_command(SetInvertMode(options.invert_colors)).await?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf)).await?;
        delay.delay_us(10_000).await;
        dcs.write_command(EnterNormalMode).await?;
        delay.delay_us(10_000).await;
        dcs.write_command(SetDisplayOn).await?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000).await;

        Ok(madctl)
    }
    
    fn clear(&mut self, color: Self::ColorFormat) -> Result<(), Error> {
        *self.framebuffer = [color.into_storage(); 240 * 135];
        
        Ok(())
    }

    fn write_pixel(&mut self, x: u16, y: u16, colors: Self::ColorFormat) -> Result<(), Error> {
        *self.framebuffer.get_mut((x + y * 240) as usize).expect("wrong index") = colors.into_storage();
        
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        let mut options = ModelOptions::with_sizes((240, 320), (240, 320));
        options.set_invert_colors(ColorInversion::Normal);

        options
    }
    
    async fn flush<DI>(&mut self, dcs: &mut AsyncDcs<DI>) -> Result<(), Error> 
    where
        DI: AsyncWriteOnlyDataCommand
    {
        dcs.write_command(WriteMemoryStart).await?;
        
        dcs.di.send_data(DataFormat::U16BE(self.framebuffer)).await?;
        
        Ok(())
    }
    
}