use crate::engine::terrain;
use crate::state::WorldParams;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

const MAP_RADIUS: f64 = 90.0;
const PIXELS_PER_BLOCK: f64 = 2.0;

pub struct Minimap {
    canvas: web_sys::HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl Minimap {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "no 2d context")?
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
            .ok_or("failed to get 2d context")?;
        Ok(Self { canvas, ctx })
    }

    pub fn render(&self, params: &WorldParams, px: f64, pz: f64, yaw_deg: i32) {
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;
        let cx = w / 2.0;
        let cy = h / 2.0;

        self.ctx.save();
        self.ctx.begin_path();
        self.ctx.arc(cx, cy, MAP_RADIUS, 0.0, std::f64::consts::TAU).ok();
        self.ctx.clip();

        let radius_blocks = (MAP_RADIUS / PIXELS_PER_BLOCK) as i32;
        for dy in -radius_blocks..=radius_blocks {
            for dx in -radius_blocks..=radius_blocks {
                let bx = px + dx as f64 * PIXELS_PER_BLOCK;
                let bz = pz + dy as f64 * PIXELS_PER_BLOCK;
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist > radius_blocks as f64 {
                    continue;
                }
                let zone = terrain::get_zone(params, bx, bz);
                let color = terrain::get_zone_color(zone);
                let sx = cx + dx as f64 * PIXELS_PER_BLOCK;
                let sy = cy + dy as f64 * PIXELS_PER_BLOCK;
                self.ctx.set_fill_style_str(
                    &format!("rgb({},{},{})",
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    )
                );
                self.ctx.fill_rect(sx, sy, PIXELS_PER_BLOCK + 0.5, PIXELS_PER_BLOCK + 0.5);
            }
        }

        self.ctx.restore();

        self.ctx.save();
        self.ctx.begin_path();
        self.ctx.arc(cx, cy, MAP_RADIUS, 0.0, std::f64::consts::TAU).ok();
        self.ctx.set_stroke_style_str("rgba(255,255,255,0.15)");
        self.ctx.set_line_width(1.5);
        self.ctx.stroke();
        self.ctx.restore();

        let angle = (yaw_deg as f64).to_radians();
        let line_len = 16.0;
        self.ctx.set_stroke_style_str("#ff3366");
        self.ctx.set_line_width(2.5);
        self.ctx.begin_path();
        self.ctx.move_to(cx, cy);
        self.ctx.line_to(cx - angle.sin() * line_len, cy - angle.cos() * line_len);
        self.ctx.stroke();

        self.ctx.set_fill_style_str("#00ffff");
        self.ctx.begin_path();
        self.ctx.arc(cx, cy, 3.0, 0.0, std::f64::consts::TAU).ok();
        self.ctx.fill();
    }
}
