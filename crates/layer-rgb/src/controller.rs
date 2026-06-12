use embassy_rp::pio::Instance;
use embassy_rp::pio_programs::ws2812::PioWs2812;
use embassy_time::Duration;
use rmk::channel::ControllerSub;
use rmk::controller::Controller;
use rmk::controller::PollingController;
use rmk::event::ControllerEvent;
use smart_leds::RGB8;

fn hsv_to_rgb8(h: u8, s: u8, v: u8) -> RGB8 {
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

fn layer_color(layer: u8) -> RGB8 {
    match layer {
        1 => RGB8::new(0, 0, 40),
        2 => RGB8::new(40, 0, 40),
        3 => RGB8::new(0, 40, 0),
        4 => RGB8::new(40, 0, 0),
        _ => RGB8::new(0, 0, 0),
    }
}

pub struct LayerRgbController<'d, P: Instance, const SM: usize, const N: usize> {
    pub ws: PioWs2812<'d, P, SM, N>,
    pub sub: ControllerSub,
    pub current_layer: u8,
    pub hue_offset: u8,
}

impl<'d, P: Instance, const SM: usize, const N: usize> Controller
    for LayerRgbController<'d, P, SM, N>
{
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        if let ControllerEvent::Layer(layer) = event {
            self.current_layer = layer;
            if layer > 0 {
                let color = layer_color(layer);
                let colors = [color; N];
                self.ws.write(&colors).await;
            }
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

impl<'d, P: Instance, const SM: usize, const N: usize> PollingController
    for LayerRgbController<'d, P, SM, N>
{
    const INTERVAL: Duration = Duration::from_millis(33);

    async fn update(&mut self) {
        if self.current_layer == 0 {
            let mut colors = [RGB8::new(0, 0, 0); N];
            for i in 0..N {
                let hue = self
                    .hue_offset
                    .wrapping_add((i as u16 * 255 / N as u16) as u8);
                colors[i] = hsv_to_rgb8(hue, 255, 32);
            }
            self.ws.write(&colors).await;
            self.hue_offset = self.hue_offset.wrapping_add(2);
        }
    }
}
