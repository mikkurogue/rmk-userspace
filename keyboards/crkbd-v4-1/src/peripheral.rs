#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;
use rmk::controller::PollingController as _;
use rmk_layer_rgb::controller::LayerRgbController;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
    #[controller(poll)]
    fn layer_rgb() -> LayerRgbController<'static, embassy_rp::peripherals::PIO1, 0, 23> {
        use embassy_rp::pio::Pio;
        use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};

        let mut pio = Pio::new(p.PIO1, rmk::split::rp::uart::IrqBinding);
        let program = PioWs2812Program::new(&mut pio.common);
        let ws: PioWs2812<'_, embassy_rp::peripherals::PIO1, 0, 23> = PioWs2812::new(
            &mut pio.common,
            pio.sm0,
            p.DMA_CH10,
            p.PIN_10,
            &program,
        );
        core::mem::forget(pio.common);

        LayerRgbController {
            ws,
            sub: defmt::unwrap!(rmk::channel::CONTROLLER_CHANNEL.subscriber()),
            current_layer: 0,
            hue_offset: 0,
        }
    }
}
