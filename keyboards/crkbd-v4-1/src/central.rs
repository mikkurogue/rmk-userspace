#![no_main]
#![no_std]

use rmk::macros::rmk_central;

const NUM_LEDS: usize = 23;

fn layer_color(layer: u8) -> smart_leds::RGB8 {
    use smart_leds::RGB8;
    match layer {
        0 => RGB8::new(0, 0, 0),   // BASE: off
        1 => RGB8::new(0, 0, 40),  // NUM: blue
        2 => RGB8::new(40, 0, 40), // SYM: purple
        3 => RGB8::new(0, 40, 0),  // EXT: green
        4 => RGB8::new(40, 0, 0),  // FUNC: red
        _ => RGB8::new(0, 0, 0),
    }
}

pub struct LayerRgbController<'d> {
    ws: embassy_rp::pio_programs::ws2812::PioWs2812<'d, embassy_rp::peripherals::PIO1, 0, { NUM_LEDS }>,
    sub: rmk::channel::ControllerSub,
}

impl<'d> rmk::controller::Controller for LayerRgbController<'d> {
    type Event = rmk::event::ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        if let rmk::event::ControllerEvent::Layer(layer) = event {
            let color = layer_color(layer);
            let colors = [color; NUM_LEDS];
            self.ws.write(&colors).await;
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

#[rmk_central]
mod keyboard_central {
    #[controller(event)]
    fn layer_rgb() -> LayerRgbController<'static> {
        use embassy_rp::pio::Pio;
        use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};

        let mut pio = Pio::new(p.PIO1, rmk::split::rp::uart::IrqBinding);
        let program = PioWs2812Program::new(&mut pio.common);
        let ws = PioWs2812::new(
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
        }
    }
}
