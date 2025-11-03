#[cfg(feature = "web")]
pub use web_thread::*;

#[cfg(not(feature = "web"))]
pub use web_thread_shim::*;
