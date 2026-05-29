#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;
use rmk::controller::PollingController as _;

const NUM_LEDS: usize = 23;

fn hsv_to_rgb(h: u8, s: u8, v: u8) -> smart_leds::RGB8 {
    use smart_leds::RGB8;
    if s == 0 {
        return RGB8::new(v, v, v);
    }
    let region = h / 43;
    let remainder = (h as u16 - region as u16 * 43) * 6;
    let p = ((v as u16 * (255 - s as u16)) >> 8) as u8;
    let q = ((v as u16 * (255 - ((s as u16 * remainder) >> 8))) >> 8) as u8;
    let t = ((v as u16 * (255 - ((s as u16 * (255 - remainder)) >> 8))) >> 8) as u8;
    match region {
        0 => RGB8::new(v, t, p),
        1 => RGB8::new(q, v, p),
        2 => RGB8::new(p, v, t),
        3 => RGB8::new(p, q, v),
        4 => RGB8::new(t, p, v),
        _ => RGB8::new(v, p, q),
    }
}

fn layer_color(layer: u8) -> smart_leds::RGB8 {
    use smart_leds::RGB8;
    match layer {
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
    current_layer: u8,
    hue_offset: u8,
}

impl<'d> rmk::controller::Controller for LayerRgbController<'d> {
    type Event = rmk::event::ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        if let rmk::event::ControllerEvent::Layer(layer) = event {
            self.current_layer = layer;
            if layer > 0 {
                let color = layer_color(layer);
                let colors = [color; NUM_LEDS];
                self.ws.write(&colors).await;
            }
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

impl<'d> rmk::controller::PollingController for LayerRgbController<'d> {
    const INTERVAL: embassy_time::Duration = embassy_time::Duration::from_millis(33);

    async fn update(&mut self) {
        if self.current_layer == 0 {
            let mut colors = [smart_leds::RGB8::new(0, 0, 0); NUM_LEDS];
            for i in 0..NUM_LEDS {
                let hue = self.hue_offset.wrapping_add((i as u16 * 255 / NUM_LEDS as u16) as u8);
                colors[i] = hsv_to_rgb(hue, 255, 32);
            }
            self.ws.write(&colors).await;
            self.hue_offset = self.hue_offset.wrapping_add(2);
        }
    }
}

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
    #[controller(poll)]
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
            current_layer: 0,
            hue_offset: 0,
        }
    }
}
