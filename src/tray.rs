//! The tray (menu bar) icon: the only visible part of the app while the window is
//! closed.

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::command::{Command, CommandSender};

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// macOS opens the menu on a click, as menu bar items do. Elsewhere a click opens the
/// window and the menu is on the right button.
const CLICK_OPENS_WINDOW: bool = !cfg!(target_os = "macos");

/// What the menu and tooltip say about the monitor. Both lines are plain text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    /// What the monitor is doing, such as `Monitoring: next test at 15:00`.
    pub headline: String,
    /// The latest result, such as `Last test 14:45: 89.36 Mbps, 4.22 ms ping`.
    pub last_test: String,
}

/// Forward tray clicks and menu choices as commands. Call once, from any thread.
pub fn listen(commands: CommandSender) {
    let menu_commands = commands.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == OPEN_ID {
            menu_commands.send(Command::Open);
        } else if event.id == QUIT_ID {
            menu_commands.send(Command::Quit);
        }
    }));
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if CLICK_OPENS_WINDOW
            && let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
        {
            commands.send(Command::Open);
        }
    }));
}

struct Parts {
    icon: TrayIcon,
    headline: MenuItem,
    last_test: MenuItem,
}

/// The icon, created the first time the event loop runs (macOS needs a running loop).
#[derive(Default)]
pub struct Tray {
    parts: Option<Parts>,
    attempted: bool,
    shown: Status,
}

impl Tray {
    /// Create the icon if that was not tried yet. Call on the main thread.
    pub fn ensure(&mut self) {
        if self.attempted {
            return;
        }
        self.attempted = true;
        match build() {
            Ok(parts) => self.parts = Some(parts),
            Err(error) => log::error!("Could not create the tray icon: {error}"),
        }
    }

    /// Make the icon respond again after an event loop run has ended.
    ///
    /// winit closes every window of the app when its event loop exits. On macOS that
    /// includes the window behind the menu bar icon: the icon stays on screen but no
    /// longer receives clicks. Hiding and showing the item makes AppKit give it a new
    /// window, in the same place. Other systems do not route the icon through an app
    /// window, so there is nothing to do there.
    pub fn revive(&self) {
        if !cfg!(target_os = "macos") {
            return;
        }
        let Some(parts) = &self.parts else {
            return;
        };
        for visible in [false, true] {
            if let Err(error) = parts.icon.set_visible(visible) {
                log::error!("Could not restore the tray icon: {error}");
            }
        }
    }

    /// Without an icon there is no way back to a closed window, so closing it has to
    /// quit the app.
    pub fn is_available(&self) -> bool {
        self.parts.is_some()
    }

    /// Show the monitor's state in the menu and the tooltip. Cheap when nothing changed.
    pub fn update(&mut self, status: Status) {
        let Some(parts) = &self.parts else {
            return;
        };
        if status == self.shown {
            return;
        }
        parts.headline.set_text(&status.headline);
        parts.last_test.set_text(&status.last_test);
        let tooltip = format!("{}\n{}", crate::APP_NAME, status.headline);
        if let Err(error) = parts.icon.set_tooltip(Some(tooltip)) {
            log::error!("Could not update the tray tooltip: {error}");
        }
        self.shown = status;
    }
}

fn build() -> Result<Parts, String> {
    // Two lines of status that cannot be clicked, then the actions.
    let headline = MenuItem::new("Starting…", false, None);
    let last_test = MenuItem::new("No tests yet", false, None);
    let menu = Menu::new();
    menu.append_items(&[
        &headline,
        &last_test,
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(OPEN_ID, format!("Open {}", crate::APP_NAME), true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(QUIT_ID, format!("Quit {}", crate::APP_NAME), true, None),
    ])
    .map_err(|error| error.to_string())?;

    let builder = TrayIconBuilder::new()
        .with_tooltip(crate::APP_NAME)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(!CLICK_OPENS_WINDOW);
    // macOS recolours a template image for light and dark menu bars.
    let builder = if cfg!(target_os = "macos") {
        builder.with_icon_templated(icon()?)
    } else {
        builder.with_icon(icon()?)
    };
    let icon = builder.build().map_err(|error| error.to_string())?;
    Ok(Parts {
        icon,
        headline,
        last_test,
    })
}

/// The icon in the form each system expects:
/// - macOS: a single-colour template image, like other menu bar items.
/// - Windows: the 32 px app icon; the taskbar scales it to 16-32 px.
/// - Linux: the 64 px app icon; the panel scales it.
fn icon() -> Result<Icon, String> {
    let (rgba, width, height) = if cfg!(target_os = "macos") {
        template_icon(&decode(include_bytes!("../docs/assets/icon-64x64.png"))?)
    } else if cfg!(target_os = "windows") {
        decode(include_bytes!("../docs/assets/icon-32x32.png"))?
    } else {
        decode(include_bytes!("../docs/assets/icon-64x64.png"))?
    };
    Icon::from_rgba(rgba, width, height).map_err(|error| error.to_string())
}

type Rgba = (Vec<u8>, u32, u32);

fn decode(png: &[u8]) -> Result<Rgba, String> {
    let image = image::load_from_memory(png)
        .map_err(|error| error.to_string())?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Ok((image.into_raw(), width, height))
}

/// Turn the app icon into a macOS template image: the coloured parts become opaque
/// black, the white parts (ring, needle) transparent. A transparent margin keeps the
/// glyph at about 16 pt in the 22 pt menu bar.
pub fn template_icon((rgba, width, height): &Rgba) -> Rgba {
    const MARGIN_PERCENT: u32 = 19;
    let margin = width * MARGIN_PERCENT / 100;
    let (out_width, out_height) = (width + 2 * margin, height + 2 * margin);
    let mut out = vec![0u8; (out_width * out_height * 4) as usize];
    for y in 0..*height {
        for x in 0..*width {
            let source = ((y * width + x) * 4) as usize;
            let pixel = &rgba[source..source + 4];
            // White has every channel high; the icon's blues have a low red channel.
            let whiteness = f32::from(pixel[0].min(pixel[1]).min(pixel[2])) / 255.0;
            let ink = ((1.0 - whiteness) / 0.75).clamp(0.0, 1.0);
            let alpha = (f32::from(pixel[3]) * ink).round() as u8;
            let target = (((y + margin) * out_width + x + margin) * 4) as usize;
            out[target + 3] = alpha;
        }
    }
    (out, out_width, out_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_icon_keeps_colour_and_drops_white() {
        // One blue pixel, one white pixel, one transparent pixel.
        let source: Rgba = (
            vec![0x34, 0x98, 0xdb, 255, 255, 255, 255, 255, 0, 0, 0, 0],
            3,
            1,
        );
        let (rgba, width, height) = template_icon(&source);
        assert_eq!((width, height), (3, 1));
        let alphas: Vec<u8> = rgba.chunks(4).map(|pixel| pixel[3]).collect();
        assert_eq!(alphas, vec![255, 0, 0]);
        // Template images are black; only alpha carries the shape.
        assert!(rgba.chunks(4).all(|pixel| pixel[..3] == [0, 0, 0]));
    }
}
