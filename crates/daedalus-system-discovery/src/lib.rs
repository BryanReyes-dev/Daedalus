//! Standard-library host provider for system and processor discovery.
//!
//! The provider targets operating systems supported by `sysinfo`. Bare-metal
//! firmware should implement `daedalus_system_info::SystemInfoProvider` using
//! its board-support package rather than depending on this crate.

mod sysinfo_provider;

pub use sysinfo_provider::SysinfoProvider;
