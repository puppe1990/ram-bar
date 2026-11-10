//! 2×2 grid template for the macOS extra (matches the RAM tab mark).
//! Black + alpha; `icon_as_template` lets the menu bar tint it.

/// Menu-bar mark at 2× (18pt).
pub const TRAY_GRID_SIZE: u32 = 36;

/// RGBA bytes of the 2×2 grid, row-major, black squares on transparent.
///
/// ```
/// let rgba = ram_bar::tray_grid::tray_grid_rgba();
/// assert_eq!(rgba.len(), 36 * 36 * 4);
/// ```
pub fn tray_grid_rgba() -> Vec<u8> {
    let size = TRAY_GRID_SIZE;
    let mut buf = vec![0u8; (size * size * 4) as usize];
    let scale = f64::from(size) / 16.0;
    // Same four rounded squares as the RAM tab SVG (viewBox 0 0 16 16).
    let cells = [(1.5, 1.5), (9.0, 1.5), (1.5, 9.0), (9.0, 9.0)];
    for (x, y) in cells {
        fill_rounded_rect(
            &mut buf,
            size,
            x * scale,
            y * scale,
            5.5 * scale,
            1.2 * scale,
        );
    }
    buf
}

fn fill_rounded_rect(buf: &mut [u8], size: u32, x: f64, y: f64, side: f64, radius: f64) {
    let right = x + side;
    let bottom = y + side;
    for py in 0..size {
        for px in 0..size {
            if !inside_rounded_rect(
                f64::from(px) + 0.5,
                f64::from(py) + 0.5,
                x,
                y,
                right,
                bottom,
                radius,
            ) {
                continue;
            }
            let i = ((py * size + px) * 4) as usize;
            buf[i] = 0;
            buf[i + 1] = 0;
            buf[i + 2] = 0;
            buf[i + 3] = 255;
        }
    }
}

fn inside_rounded_rect(
    cx: f64,
    cy: f64,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    radius: f64,
) -> bool {
    if cx < left || cx >= right || cy < top || cy >= bottom {
        return false;
    }
    let ix0 = left + radius;
    let iy0 = top + radius;
    let ix1 = right - radius;
    let iy1 = bottom - radius;
    if cx >= ix0 && cx < ix1 {
        return true;
    }
    if cy >= iy0 && cy < iy1 {
        return true;
    }
    let nx = if cx < ix0 { ix0 } else { ix1 };
    let ny = if cy < iy0 { iy0 } else { iy1 };
    let dx = cx - nx;
    let dy = cy - ny;
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * TRAY_GRID_SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn rgba_is_36_square() {
        let buf = tray_grid_rgba();
        assert_eq!(buf.len(), (TRAY_GRID_SIZE * TRAY_GRID_SIZE * 4) as usize);
    }

    #[test]
    fn corners_are_transparent() {
        let buf = tray_grid_rgba();
        let last = TRAY_GRID_SIZE - 1;
        assert_eq!(pixel(&buf, 0, 0)[3], 0);
        assert_eq!(pixel(&buf, last, 0)[3], 0);
        assert_eq!(pixel(&buf, 0, last)[3], 0);
        assert_eq!(pixel(&buf, last, last)[3], 0);
    }

    #[test]
    fn four_cells_are_opaque_black() {
        let buf = tray_grid_rgba();
        let size = TRAY_GRID_SIZE;
        let centers = [
            (size / 4, size / 4),
            (size * 3 / 4, size / 4),
            (size / 4, size * 3 / 4),
            (size * 3 / 4, size * 3 / 4),
        ];
        for (x, y) in centers {
            assert_eq!(pixel(&buf, x, y), [0, 0, 0, 255], "cell at {x},{y}");
        }
    }

    #[test]
    fn gap_at_center_is_transparent() {
        let buf = tray_grid_rgba();
        let mid = TRAY_GRID_SIZE / 2;
        assert_eq!(pixel(&buf, mid, mid)[3], 0);
    }

    #[test]
    fn ink_is_only_black() {
        let buf = tray_grid_rgba();
        for chunk in buf.chunks_exact(4) {
            if chunk[3] == 0 {
                continue;
            }
            assert_eq!(chunk, [0, 0, 0, 255]);
        }
    }
}
