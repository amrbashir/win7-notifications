// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Send Windows 10 styled notifications on Windows 7.
//!
//! # Note:
//!
//! This crate requires a win32 event loop to be running on the thread, otherwise the notification will close immediately,
//! it is recommended to use it with other win32 event loop crates like [winit](https://docs.rs/winit) or just use your own win32 event loop.
//!
//! # Examples
//!
//! # Example 1: Simple Notification
//!
//! ```no_run
//! # use win7_notifications::*;
//! # let icon = &[];
//! Notification::new()
//!     .appname("App name")
//!     .summary("Critical Error")
//!     .body("Just kidding, this is just the notification example.")
//!     .icon(icon.to_vec(), 32, 32)
//!     .timeout(Timeout::Default) // 5000 milliseconds
//!     .show().unwrap();
//! ```
//!
//! # Example 2: Presistent Notification
//!
//! ```no_run
//! # use win7_notifications::*;
//! # let icon = &[];
//! Notification::new()
//!     .appname("App name")
//!     .summary("Critical Error")
//!     .body("Just kidding, this is just the notification example.")
//!     .icon(icon.to_vec(), 32, 32)
//!     .timeout(Timeout::Never)
//!     .show().unwrap();
//! ```
//!

mod notification;
mod timeout;
mod util;

pub use crate::{notification::Notification, timeout::Timeout};
