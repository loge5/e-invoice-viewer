//! Remembering the window size safely (spec 0015).
//!
//! The normal size (neither maximized nor tiled) and the maximized state are
//! stored in the app's configuration directory. A stored size is applied only
//! if it is plausible on the current screens, so that a damaged value can
//! never make the window unusable.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use crate::config_store::{self, LOG_DOMAIN};

/// Size of a new window without a plausible stored size.
pub const DEFAULT_SIZE: (i32, i32) = (1200, 800);
/// The window cannot be made smaller than this (0007).
pub const MINIMUM_SIZE: (i32, i32) = (900, 600);

/// What is remembered of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowState {
    /// The normal size in logical pixels.
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

/// The state a new window opens with. `stored` is applied if plausible:
/// at least the minimum size and at most the largest monitor (logical
/// pixels). Otherwise the default size is used, reduced to the largest
/// monitor, but never below the minimum size.
pub fn initial_state(
    stored: Option<WindowState>,
    largest_monitor: Option<(i32, i32)>,
) -> WindowState {
    let (min_width, min_height) = MINIMUM_SIZE;
    let fits = |width: i32, height: i32| {
        width >= min_width
            && height >= min_height
            && largest_monitor
                .is_none_or(|(max_width, max_height)| width <= max_width && height <= max_height)
    };
    match stored {
        Some(stored) if fits(stored.width, stored.height) => stored,
        _ => {
            let (mut width, mut height) = DEFAULT_SIZE;
            if let Some((max_width, max_height)) = largest_monitor {
                width = width.min(max_width);
                height = height.min(max_height);
            }
            WindowState {
                width: width.max(min_width),
                height: height.max(min_height),
                maximized: stored.is_some_and(|stored| stored.maximized),
            }
        }
    }
}

/// Follows the normal size of a window and saves it with the maximized
/// state when the window is closed.
#[derive(Clone)]
pub struct SizeKeeper {
    window: adw::ApplicationWindow,
    /// The last size while neither maximized, fullscreen nor tiled.
    normal_size: Rc<Cell<(i32, i32)>>,
}

impl SizeKeeper {
    /// The size and state a new window opens with: the stored one if it
    /// is plausible on the connected monitors.
    pub fn initial_state() -> WindowState {
        let (stored, warning) = load(&store_path());
        if let Some(warning) = warning {
            glib::g_warning!(LOG_DOMAIN, "{warning}");
        }
        initial_state(stored, largest_monitor())
    }

    /// Starts following `window`, which opened with `state`.
    pub fn attach(window: &adw::ApplicationWindow, state: WindowState) -> Self {
        let keeper = Self {
            window: window.clone(),
            normal_size: Rc::new(Cell::new((state.width, state.height))),
        };
        let this = keeper.clone();
        let update = move || {
            let tiled = this
                .window
                .surface()
                .and_downcast::<gtk::gdk::Toplevel>()
                .is_some_and(|toplevel| {
                    toplevel.state().intersects(
                        gtk::gdk::ToplevelState::TILED
                            | gtk::gdk::ToplevelState::TOP_TILED
                            | gtk::gdk::ToplevelState::BOTTOM_TILED
                            | gtk::gdk::ToplevelState::LEFT_TILED
                            | gtk::gdk::ToplevelState::RIGHT_TILED,
                    )
                });
            if !this.window.is_maximized() && !this.window.is_fullscreen() && !tiled {
                this.normal_size.set(this.window.default_size());
            }
        };
        let width = update.clone();
        window.connect_default_width_notify(move |_| width());
        window.connect_default_height_notify(move |_| update());

        let this = keeper.clone();
        window.connect_close_request(move |_| {
            this.save();
            glib::Propagation::Proceed
        });
        keeper
    }

    /// Stores the normal size and the maximized state.
    pub fn save(&self) {
        let (width, height) = self.normal_size.get();
        let state = WindowState {
            width,
            height,
            maximized: self.window.is_maximized(),
        };
        if let Err(error) = save(&store_path(), state) {
            glib::g_warning!(LOG_DOMAIN, "window size not saved: {error}");
        }
    }
}

/// Size of the largest connected monitor in logical pixels.
fn largest_monitor() -> Option<(i32, i32)> {
    let monitors = gtk::gdk::Display::default()?.monitors();
    (0..monitors.n_items())
        .filter_map(|index| monitors.item(index).and_downcast::<gtk::gdk::Monitor>())
        .map(|monitor| {
            let geometry = monitor.geometry();
            (geometry.width(), geometry.height())
        })
        .max_by_key(|(width, height)| i64::from(*width) * i64::from(*height))
}

const GROUP: &str = "Window";

fn unusable(path: &Path, reason: &str) -> (Option<WindowState>, Option<String>) {
    (
        None,
        Some(format!("window size {} ignored: {reason}", path.display())),
    )
}

/// Where the window state is stored.
pub fn store_path() -> PathBuf {
    config_store::path("window")
}

/// Reads the stored state. Returns the state and, if the store exists but
/// cannot be used, a warning to log.
pub fn load(path: &Path) -> (Option<WindowState>, Option<String>) {
    let key_file = match config_store::read(path) {
        Ok(Some(key_file)) => key_file,
        Ok(None) => return (None, None),
        Err(reason) => return unusable(path, &reason),
    };
    let state = (|| {
        Ok::<_, glib::Error>(WindowState {
            width: key_file.integer(GROUP, "width")?,
            height: key_file.integer(GROUP, "height")?,
            maximized: key_file.boolean(GROUP, "maximized")?,
        })
    })();
    match state {
        Ok(state) => (Some(state), None),
        Err(error) => unusable(path, error.message()),
    }
}

/// Stores the state; the previous file is replaced atomically.
pub fn save(path: &Path, state: WindowState) -> Result<(), String> {
    let key_file = glib::KeyFile::new();
    key_file.set_integer(GROUP, "width", state.width);
    key_file.set_integer(GROUP, "height", state.height);
    key_file.set_boolean(GROUP, "maximized", state.maximized);
    config_store::write(path, &key_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UHD: Option<(i32, i32)> = Some((3840, 2160));

    fn state(width: i32, height: i32) -> WindowState {
        WindowState {
            width,
            height,
            maximized: false,
        }
    }

    fn size(state: WindowState) -> (i32, i32) {
        (state.width, state.height)
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("e-invoice-viewer-tests")
            .join(format!("window-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    // AC4
    #[test]
    fn plausibility_table() {
        for (stored, expected) in [
            ((1, 38), (1200, 800)),
            ((-500, 700), (1200, 800)),
            ((800, 500), (1200, 800)),
            ((9000, 9000), (1200, 800)),
            ((1400, 900), (1400, 900)),
        ] {
            assert_eq!(
                size(initial_state(Some(state(stored.0, stored.1)), UHD)),
                expected,
                "{stored:?}"
            );
        }
        assert_eq!(size(initial_state(None, UHD)), (1200, 800));
    }

    // AC4
    #[test]
    fn scaled_and_small_monitors() {
        // 3840 × 2160 at 200 % scaling is 1920 × 1080 logical pixels.
        assert_eq!(
            size(initial_state(Some(state(2400, 1200)), Some((1920, 1080)))),
            (1200, 800)
        );
        assert_eq!(size(initial_state(None, Some((1024, 768)))), (1024, 768));
        assert_eq!(size(initial_state(None, Some((800, 600)))), (900, 600));
        // Without monitor information only the minimum size is checked.
        assert_eq!(
            size(initial_state(Some(state(1400, 900)), None)),
            (1400, 900)
        );
    }

    // AC1, AC2
    #[test]
    fn settings_round_trip() {
        let path = scratch_dir("round-trip").join("nested").join("window");
        for saved in [
            state(1400, 900),
            WindowState {
                maximized: true,
                ..state(1000, 700)
            },
        ] {
            save(&path, saved).expect("save");
            assert_eq!(load(&path), (Some(saved), None));
        }
        // A maximized window opens maximized with its normal size.
        let maximized = initial_state(load(&path).0, UHD);
        assert!(maximized.maximized);
        assert_eq!(size(maximized), (1000, 700));
    }

    // AC5
    #[test]
    fn broken_settings_use_default_with_one_warning() {
        let dir = scratch_dir("broken");
        assert_eq!(load(&dir.join("missing")), (None, None));
        for (name, content) in [
            ("empty", ""),
            ("truncated", "[Window]\nwidth=14"),
            ("garbage", "\u{0}\u{1} no key file"),
            (
                "not-a-number",
                "[Window]\nwidth=breit\nheight=900\nmaximized=false\n",
            ),
        ] {
            let path = dir.join(name);
            std::fs::write(&path, content).expect("write");

            let (stored, warning) = load(&path);

            assert_eq!(stored, None, "{name}");
            assert!(warning.is_some(), "{name}");
            assert_eq!(size(initial_state(stored, UHD)), (1200, 800));
        }
    }

    // AC5
    #[test]
    fn unknown_entries_are_ignored() {
        let path = scratch_dir("unknown").join("window");
        std::fs::write(
            &path,
            "[Window]\nwidth=1400\nheight=900\nmaximized=false\nzoom=150\n\n[Later]\nview=xml\n",
        )
        .expect("write");

        assert_eq!(load(&path), (Some(state(1400, 900)), None));
    }

    // AC5, AC7
    #[test]
    fn settings_live_in_app_config_dir_apart_from_recent_list() {
        let dir = gtk::glib::user_config_dir().join("e-invoice-viewer");
        assert_eq!(store_path(), dir.join("window"));
        assert_ne!(store_path(), crate::recent::store_path());
    }
}
