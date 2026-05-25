use crate::engine::terrain;
use crate::state::WorldParams;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

const MAP_RADIUS: f64 = 90.0;
const PIXELS_PER_BLOCK: f64 = 2.0;

const FULL_PIXELS_PER_BLOCK: f64 = 4.0;
const FULL_MAP_SIZE: f64 = 800.0; // canvas logical size

/// Returns (blocks_visible, offset) to center the map
fn full_map_blocks() -> i32 {
    (FULL_MAP_SIZE / FULL_PIXELS_PER_BLOCK) as i32
}

pub fn world_to_canvas(wx: f64, wz: f64, px: f64, pz: f64, cw: f64, ch: f64) -> (f64, f64) {
    let cx = cw / 2.0 + (wx - px) * FULL_PIXELS_PER_BLOCK;
    let cy = ch / 2.0 + (wz - pz) * FULL_PIXELS_PER_BLOCK;
    (cx, cy)
}

pub fn canvas_to_world(cx: f64, cy: f64, px: f64, pz: f64, cw: f64, ch: f64) -> (f64, f64) {
    let wx = px + (cx - cw / 2.0) / FULL_PIXELS_PER_BLOCK;
    let wz = pz + (cy - ch / 2.0) / FULL_PIXELS_PER_BLOCK;
    (wx, wz)
}

pub fn render_full_map(
    ctx: &CanvasRenderingContext2d,
    cw: f64,
    ch: f64,
    params: &WorldParams,
    px: f64,
    pz: f64,
    yaw_deg: i32,
    waypoints: &[(f64, f64, f64, String)],
) {
    let half_blocks = full_map_blocks() / 2;

    // Background
    ctx.set_fill_style_str("#0a0a12");
    ctx.fill_rect(0.0, 0.0, cw, ch);

    // Terrain zone grid
    for dy in -half_blocks..=half_blocks {
        for dx in -half_blocks..=half_blocks {
            let wx = px + dx as f64 * FULL_PIXELS_PER_BLOCK;
            let wz = pz + dy as f64 * FULL_PIXELS_PER_BLOCK;
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > half_blocks as f64 {
                continue;
            }
            let zone = terrain::get_zone(params, wx, wz);
            let color = terrain::get_zone_color(zone);
            let sx = cw / 2.0 + dx as f64 * FULL_PIXELS_PER_BLOCK;
            let sy = ch / 2.0 + dy as f64 * FULL_PIXELS_PER_BLOCK;
            ctx.set_fill_style_str(&format!("rgb({},{},{})",
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
            ));
            ctx.fill_rect(sx, sy, FULL_PIXELS_PER_BLOCK + 0.5, FULL_PIXELS_PER_BLOCK + 0.5);
        }
    }

    // Grid lines (every 10 blocks)
    ctx.set_stroke_style_str("rgba(255,255,255,0.04)");
    ctx.set_line_width(0.5);
    for i in (-half_blocks..=half_blocks).step_by(10) {
        let sx = cw / 2.0 + i as f64 * FULL_PIXELS_PER_BLOCK;
        ctx.begin_path();
        ctx.move_to(sx, 0.0);
        ctx.line_to(sx, ch);
        ctx.stroke();
        let sy = ch / 2.0 + i as f64 * FULL_PIXELS_PER_BLOCK;
        ctx.begin_path();
        ctx.move_to(0.0, sy);
        ctx.line_to(cw, sy);
        ctx.stroke();
    }

    // Axis labels
    ctx.set_fill_style_str("rgba(255,255,255,0.12)");
    ctx.set_font("10px monospace");
    for i in (-half_blocks..=half_blocks).step_by(20) {
        if i == 0 { continue; }
        let world_coord = (i as f64 * FULL_PIXELS_PER_BLOCK) as i32;
        let sx = cw / 2.0 + i as f64 * FULL_PIXELS_PER_BLOCK;
        ctx.fill_text(&format!("{}", world_coord), sx + 2.0, ch - 4.0).ok();
        let sy = ch / 2.0 + i as f64 * FULL_PIXELS_PER_BLOCK;
        ctx.fill_text(&format!("{}", world_coord), 4.0, sy + 10.0).ok();
    }

    // Waypoints
    for (wx, _, wz, name) in waypoints {
        let (sx, sy) = world_to_canvas(*wx, *wz, px, pz, cw, ch);
        if sx < 0.0 || sx > cw || sy < 0.0 || sy > ch {
            continue;
        }

        // Pin shadow
        ctx.set_fill_style_str("rgba(0,0,0,0.4)");
        ctx.begin_path();
        ctx.arc(sx + 1.0, sy + 2.0, 5.0, 0.0, std::f64::consts::TAU).ok();
        ctx.fill();

        // Pin body
        ctx.set_fill_style_str("#ff3366");
        ctx.begin_path();
        ctx.move_to(sx, sy - 8.0);
        ctx.line_to(sx - 5.0, sy);
        ctx.line_to(sx + 5.0, sy);
        ctx.close_path();
        ctx.fill();

        // Pin circle
        ctx.set_fill_style_str("#ffffff");
        ctx.begin_path();
        ctx.arc(sx, sy - 5.0, 2.5, 0.0, std::f64::consts::TAU).ok();
        ctx.fill();

        // Name label
        ctx.set_fill_style_str("rgba(255,255,255,0.7)");
        ctx.set_font("10px monospace");
        ctx.fill_text(name, sx + 8.0, sy + 3.0).ok();
    }

    // Player position (crosshair)
    let pcx = cw / 2.0;
    let pcy = ch / 2.0;

    // Outer ring
    ctx.set_stroke_style_str("rgba(0,255,255,0.3)");
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.arc(pcx, pcy, 10.0, 0.0, std::f64::consts::TAU).ok();
    ctx.stroke();

    // Cross
    let cross = 5.0;
    ctx.set_stroke_style_str("#00ffff");
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(pcx - cross, pcy);
    ctx.line_to(pcx + cross, pcy);
    ctx.move_to(pcx, pcy - cross);
    ctx.line_to(pcx, pcy + cross);
    ctx.stroke();

    // Direction line
    let angle = (yaw_deg as f64).to_radians();
    let line_len = 20.0;
    ctx.set_stroke_style_str("#ff3366");
    ctx.set_line_width(2.5);
    ctx.begin_path();
    ctx.move_to(pcx, pcy);
    ctx.line_to(pcx - angle.sin() * line_len, pcy - angle.cos() * line_len);
    ctx.stroke();

    // Center dot
    ctx.set_fill_style_str("#ffffff");
    ctx.begin_path();
    ctx.arc(pcx, pcy, 2.0, 0.0, std::f64::consts::TAU).ok();
    ctx.fill();

    // Compass rose (top right)
    let compass_x = cw - 80.0;
    let compass_y = 60.0;
    ctx.set_fill_style_str("rgba(255,255,255,0.15)");
    ctx.begin_path();
    ctx.arc(compass_x, compass_y, 20.0, 0.0, std::f64::consts::TAU).ok();
    ctx.fill();

    ctx.set_fill_style_str("rgba(255,255,255,0.5)");
    ctx.set_font("14px monospace");
    ctx.set_text_align("center");
    ctx.fill_text("N", compass_x, compass_y - 24.0).ok();
    ctx.fill_text("S", compass_x, compass_y + 32.0).ok();
    ctx.set_text_align("right");
    ctx.fill_text("O", compass_x - 28.0, compass_y + 5.0).ok();
    ctx.set_text_align("left");
    ctx.fill_text("E", compass_x + 28.0, compass_y + 5.0).ok();
    ctx.set_text_align("start");

    // Info text (top left)
    ctx.set_fill_style_str("rgba(255,255,255,0.3)");
    ctx.set_font("11px monospace");
    ctx.fill_text(&format!("📍 {:.0}, {:.0}", px, pz), 12.0, 24.0).ok();
    ctx.fill_text(&format!("📍 Waypoints: {}", waypoints.len()), 12.0, 40.0).ok();
}

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
