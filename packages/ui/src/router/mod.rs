pub mod common;

#[cfg(feature = "mobile")]
mod mobile;
#[cfg(all(feature = "desktop", not(feature = "mobile")))]
mod desktop;

#[cfg(feature = "mobile")]
pub use mobile::*;

#[cfg(all(feature = "desktop", not(feature = "mobile")))]
pub use desktop::*;

pub use common::AppRoot;
