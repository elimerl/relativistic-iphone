mod font;
mod wsserver;

use font::FONT_8X8;
use glam::{vec3, Vec3};
use minifb::{Key, Window, WindowOptions};
use wsserver::{server, Acceleration};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

fn main() {
    let c = 100.0; // speed of light in m/s

    let (tx, rx) = std::sync::mpsc::channel::<Acceleration>();
    std::thread::spawn(move || {
        server(tx);
    });
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "relativistic iphone",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 40.0)));
    let mut plot_data: Vec<u8> = Vec::new();
    let mut velocity = Vec3::ZERO;
    let mut peak_lorentz = 1.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Ok(v) = rx.try_recv() {
            velocity = vec3(v.x, v.y, v.z);
            plot_data.push((velocity.length() * 10.0) as u8);
        } else {
            plot_data.push(0);
        }
        let lorentz_factor = 1.0 / (1.0 - (velocity.length() / c).powf(2.0)).sqrt();
        if lorentz_factor > peak_lorentz {
            peak_lorentz = lorentz_factor;
        }
        buffer.fill(0);
        draw_text(
            &mut buffer,
            &format!(
                "v={:.2}m/s\nlorentz = {:.2}\n1/lorentz = {:.2}\npeak lorentz = {:.2}",
                velocity.length(),
                lorentz_factor,
                lorentz_factor.recip(),
                peak_lorentz
            ),
            (4, 4),
            0xffffffff,
        );

        for (i, data) in plot_data.iter().rev().take(200).rev().enumerate() {
            set_pixel(
                &mut buffer,
                i,
                (199 - *data) as usize,
                0xffff0000 + ((*data as u32) << 2),
            );
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn set_pixel(buffer: &mut [u32], x: usize, y: usize, color: u32) {
    buffer[y * WIDTH + x] = color;
}

fn line(buffer: &mut [u32], mut start: (usize, usize), end: (usize, usize), color: u32) {
    let dx = (end.0).abs_diff(start.0) as isize;
    let sx = if start.0 < end.0 { 1isize } else { -1isize };
    let dy = -((end.1).abs_diff(start.1) as isize);
    let sy = if start.1 < end.1 { 1isize } else { -1isize };
    let mut error = dx + dy;

    loop {
        set_pixel(buffer, start.0, start.1, color);
        if start == end {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if start.0 == end.0 {
                break;
            }
            error += dy;
            start.0 = (start.0 as isize + sx) as usize;
        }
        if e2 <= dx {
            if start.1 == end.1 {
                break;
            }
            error += dx;
            start.1 = (start.1 as isize + sy) as usize;
        }
    }
}

fn draw_char(buffer: &mut [u32], char: char, top_left: (usize, usize), color: u32) {
    let font8x8 = FONT_8X8[char as usize];
    for (row, char_font) in font8x8.iter().enumerate() {
        for col in 0..8 {
            if (char_font >> col) & 0x1 != 0 {
                set_pixel(buffer, top_left.0 + col, top_left.1 + row, color);
            }
        }
    }
}

fn draw_text(buffer: &mut [u32], text: &str, mut top_left: (usize, usize), color: u32) {
    let orig_x = top_left.0;
    for c in text.chars() {
        if c == '\n' {
            top_left.0 = orig_x;
            top_left.1 += 9;
            continue;
        }
        draw_char(buffer, c, top_left, color);
        top_left.0 += 9;
    }
}
