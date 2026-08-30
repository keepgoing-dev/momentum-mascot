//! Where the popover hangs from.
//!
//! One rule serves both entry points, because the two are the same shape of problem: a small
//! panel hung off something the user just clicked, kept on that thing's own display.
//!
//! The display part is the whole reason this is not three lines inline. macOS moves the menu
//! bar's status items to whichever display is active, so the tray icon is not reliably on the
//! same screen as the pet, and a popover anchored to the tray after a click on the pet opened
//! on the *other monitor* - clicked in the bottom-right of one screen, answered in the top-left
//! of another. Anchoring to whatever was actually clicked is what fixes that, and it needs the
//! work area of that thing's display rather than of the primary one.

/// A rectangle in physical pixels with a top-left origin, which is the space every window
/// position in this app is in. AppKit's own y-up, primary-origin space is converted away at
/// the edges (`tray::rect`) rather than carried around.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// The popover's top-left: centred on `anchor`, below it if the anchor sits in the top half of
/// the display and above it otherwise, and clamped so the whole panel stays inside `area`.
///
/// The half-and-half rule is what lets the tray icon and the pet share this function. The tray
/// is in the menu bar, so it opens downwards as it always has; the pet is usually in a bottom
/// corner, where opening downwards would put the panel off the bottom of the screen.
///
/// `area` is the work area, which excludes the menu bar and the Dock - and the tray icon is
/// *inside* the menu bar, so the anchor is legitimately outside the area it is clamped into.
/// That is fine and is why the clamp is applied to the result rather than to the anchor.
pub fn anchored(anchor: Rect, size: (f64, f64), area: Rect, gap: f64) -> (f64, f64) {
    let x = clamp(anchor.center().0 - size.0 / 2.0, area.x + gap, area.x + area.w - size.0 - gap);
    let opens_down = anchor.center().1 < area.center().1;
    let y = if opens_down {
        anchor.y + anchor.h + gap
    } else {
        anchor.y - gap - size.1
    };
    (x, clamp(y, area.y + gap, area.y + area.h - size.1 - gap))
}

/// `f64::clamp` panics when the bounds cross, which they do the moment a panel is wider than
/// the display it is on. The low bound wins there: better flush to the top-left and partly off
/// screen than off screen entirely.
fn clamp(v: f64, low: f64, high: f64) -> f64 {
    v.max(low).min(high.max(low))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The author's own desk, in physical pixels: a 1600x1000 laptop display as primary, and a
    /// 3360x1418 display arranged above it, which puts the second one at negative y.
    const LAPTOP: Rect = Rect { x: 0.0, y: 62.0, w: 3200.0, h: 1938.0 };
    const ABOVE: Rect = Rect { x: -1760.0, y: -2836.0, w: 6720.0, h: 2836.0 };
    const SIZE: (f64, f64) = (704.0, 916.0);
    const GAP: f64 = 12.0;

    #[test]
    fn the_tray_icon_opens_downwards_from_the_menu_bar() {
        let tray = Rect::new(1140.0, 0.0, 68.0, 60.0);
        let (x, y) = anchored(tray, SIZE, LAPTOP, GAP);
        // Not `0 + 60 + GAP`: that lands 2px above the work area's own inset top, and the
        // clamp lifts it. The two are within a pixel of each other by construction, the menu
        // bar being about as tall as the icon in it.
        assert_eq!(y, LAPTOP.y + GAP, "hangs below the icon, clear of the menu bar");
        assert_eq!(x, 822.0, "centred on the icon");
    }

    #[test]
    fn the_pet_in_a_bottom_corner_opens_upwards() {
        let pet = Rect::new(3032.0, 1832.0, 128.0, 128.0);
        let (_, y) = anchored(pet, SIZE, LAPTOP, GAP);
        assert_eq!(y, 1832.0 - GAP - SIZE.1, "sits above the pet rather than off the bottom");
    }

    #[test]
    fn the_pet_dragged_to_a_top_corner_opens_downwards() {
        let pet = Rect::new(40.0, 102.0, 128.0, 128.0);
        let (_, y) = anchored(pet, SIZE, LAPTOP, GAP);
        assert_eq!(y, 102.0 + 128.0 + GAP);
    }

    /// The bug this file exists for. The anchor decides the display, so a pet on the laptop
    /// keeps its popover on the laptop no matter where the menu bar has wandered to.
    #[test]
    fn the_popover_stays_on_the_display_its_anchor_is_on() {
        let pet = Rect::new(3032.0, 1832.0, 128.0, 128.0);
        let (x, y) = anchored(pet, SIZE, LAPTOP, GAP);
        assert!(LAPTOP.contains(x, y) && LAPTOP.contains(x + SIZE.0 - 1.0, y + SIZE.1 - 1.0));

        let tray = Rect::new(3440.0, -2836.0, 68.0, 60.0);
        let (x, y) = anchored(tray, SIZE, ABOVE, GAP);
        assert!(ABOVE.contains(x, y) && ABOVE.contains(x + SIZE.0 - 1.0, y + SIZE.1 - 1.0));
    }

    #[test]
    fn a_corner_anchor_pulls_the_panel_back_inside_the_work_area() {
        let pet = Rect::new(3032.0, 1832.0, 128.0, 128.0);
        let (x, _) = anchored(pet, SIZE, LAPTOP, GAP);
        assert_eq!(x, LAPTOP.w - SIZE.0 - GAP, "would have run off the right edge");

        let pet = Rect::new(40.0, 1832.0, 128.0, 128.0);
        let (x, _) = anchored(pet, SIZE, LAPTOP, GAP);
        assert_eq!(x, LAPTOP.x + GAP);
    }

    /// A display shorter than the popover crosses the clamp's bounds, which is a panic in
    /// `f64::clamp` and a window nobody can reach if the high bound wins.
    #[test]
    fn a_panel_taller_than_the_display_lands_at_the_top_of_it() {
        let tiny = Rect::new(0.0, 0.0, 800.0, 600.0);
        let (x, y) = anchored(Rect::new(0.0, 0.0, 68.0, 60.0), SIZE, tiny, GAP);
        assert_eq!((x, y), (GAP, GAP));
    }
}
