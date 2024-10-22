// Note: there's one copy of this static per shared object on purpose — that's the one
// static we DON'T want to deduplicate.
#[used]
static SHARED_OBJECT_ID_REF: u64 = 0;

/// Returns a unique identifier for the current shared object.
pub fn shared_object_id() -> u64 {
    &SHARED_OBJECT_ID_REF as *const _ as u64
}

/// The name of this package
pub static mut SO_NAME: &str = "<unknown>";

/// Initializes the `SO_NAME` static to make logs more readable.
#[macro_export]
macro_rules! init {
    () => {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| unsafe {
            $crate::SO_NAME = Box::leak(Box::new(format!("{: <12}", env!("CARGO_PKG_NAME"))));
        });
    };
}

/// Prints a message, prefixed with a cycling millisecond timestamp (wraps at 99999),
/// a colorized shared object id, a colorized thread name+id, and the given message.
#[macro_export]
#[cfg(feature = "print")]
macro_rules! soprintln {
    ($($arg:tt)*) => {
        {
            use std::sync::atomic::{AtomicBool, Ordering};
            static ENV_CHECKED: std::sync::Once = std::sync::Once::new();
            static SHOULD_PRINT: AtomicBool = AtomicBool::new(false);
            ENV_CHECKED.call_once(|| {
                let should_print = std::env::var("SOPRINTLN").map(|v| v == "1").unwrap_or(false);
                SHOULD_PRINT.store(should_print, Ordering::Relaxed);
            });

            if SHOULD_PRINT.load(Ordering::Relaxed) {
                // this formatting is terribly wasteful — PRs welcome

                let so_id = $crate::shared_object_id();
                let so_mode_and_id = $crate::Beacon::new(unsafe { $crate::SO_NAME }, so_id).show_val(false);
                let curr_thread = std::thread::current();
                let tid = format!("{:?}", curr_thread.id());
                // strip `ThreadId(` prefix
                let tid = tid.strip_prefix("ThreadId(").unwrap_or(&tid);
                // strip `)` suffix
                let tid = tid.strip_suffix(")").unwrap_or(&tid);
                // parse tid as u64
                let tid = tid.parse::<u64>().unwrap_or(0);

                let thread_name = curr_thread.name().unwrap_or("<unnamed>").trim();
                let thread_name = if thread_name.len() > 20 {
                    format!("{}...{}", &thread_name[..8], &thread_name[thread_name.len() - 9..])
                } else {
                    format!("{: <20}", thread_name)
                };
                let thread = $crate::Beacon::new(&thread_name, tid);

                let timestamp = ::std::time::SystemTime::now().duration_since(::std::time::UNIX_EPOCH).unwrap().as_millis() % 99999;

                // compute the 24-bit ANSI color of the timestamp based on its value between 0 and 99999
                let hue = (timestamp % 1000) as f64 / 999.0 * 360.0;
                let saturation = 40.0;
                let lightness = 100.0;
                let (fg_r, fg_g, fg_b) = $crate::hsl_to_rgb(hue, saturation, lightness);
                let (bg_r, bg_g, bg_b) = $crate::hsl_to_rgb(hue, saturation * 0.8, lightness * 0.5);

                // coloring the timestamp helps spot time gaps in the log

                // FIXME: this is probably not necessary, but without it, rustc complains about
                // capturing variables in format_args?
                let msg = format!($($arg)*);
                eprintln!("⏰\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{:05}\x1b[0m 📦{so_mode_and_id} 🧵{thread} {msg}", bg_r, bg_g, bg_b, fg_r, fg_g, fg_b, timestamp);
            }
        }
    };
}

/// `soprintln!` prints a message prefixed by a truncated timestamp, shared object ID and thread ID.
///
/// It is costly, which is why it's behind a cargo feature AND an environment variable.
///
/// To see soprintln output, enable the `soprintln` cargo feature, and set the `SOPRINTLN`
/// environment variable to `1`.
#[macro_export]
#[cfg(not(feature = "print"))]
macro_rules! soprintln {
    ($($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

/// A `u64` whose 24-bit ANSI color is determined by its value.
///
/// Used by the [`soprintln`] macro to visually distinguish shared objects and threads.
pub struct Beacon<'a> {
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    name: &'a str,
    val: u64,
    show_val: bool,
}

impl<'a> Beacon<'a> {
    /// Creates a new `Beacon` from a pointer.
    pub fn from_ptr<T>(name: &'a str, ptr: *const T) -> Self {
        Self::new(name, ptr as u64)
    }

    /// Creates a new `Beacon` from a reference.
    pub fn from_ref<T>(name: &'a str, r: &T) -> Self {
        Self::new(name, r as *const T as u64)
    }

    /// Creates a new `Beacon` with the given extra string and value.
    pub fn new(name: &'a str, u: u64) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash(x: u64) -> u64 {
            let mut hasher = DefaultHasher::new();
            x.hash(&mut hasher);
            hasher.finish()
        }

        let hashed_float = (hash(u) as f64) / (u64::MAX as f64);
        let h = hashed_float * 360.0;
        let s = 37.0;
        let l = 91.0;

        let fg = hsl_to_rgb(h, s, l);
        let bg = hsl_to_rgb(h, s * 0.8, l * 0.6);

        Self {
            fg,
            bg,
            name,
            val: u,
            show_val: true,
        }
    }

    pub fn show_val(mut self, show_val: bool) -> Self {
        self.show_val = show_val;
        self
    }
}

/// Converts a hue, saturation, and lightness to an RGB color.
///
/// h is in [0, 360)
/// s is in [0, 100]
/// l is in [0, 100]
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h * 6.0) as u8 {
        0 | 6 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

impl<'a> std::fmt::Display for Beacon<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_val {
            write!(
                f,
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}·{:03x}\x1b[0m",
                self.bg.0,
                self.bg.1,
                self.bg.2,
                self.fg.0,
                self.fg.1,
                self.fg.2,
                self.name,
                self.val
            )
        } else {
            write!(
                f,
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}\x1b[0m",
                self.bg.0, self.bg.1, self.bg.2, self.fg.0, self.fg.1, self.fg.2, self.name
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon() {
        crate::init!();
        for i in 0..128 {
            if i == 64 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
            let b = Beacon::new("test", 0x12345678 + i);
            soprintln!("{b}");
        }
    }

    #[test]
    fn test_soprintln() {
        crate::init!();
        soprintln!("test");
    }
}
