// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::Cell, ffi::OsStr, iter::once, os::windows::prelude::OsStrExt, ptr};

use windows_sys::Win32::{
  Foundation::*,
  Graphics::Gdi::*,
  System::Com::*,
  UI::WindowsAndMessaging::{self as w32wm, *},
};

use crate::definitions::*;

pub fn current_exe_name() -> String {
  std::env::current_exe()
    .unwrap()
    .file_name()
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned()
}
pub fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
  string.as_ref().encode_wide().chain(once(0)).collect()
}

/// Implementation of the `RGB` macro.
#[allow(non_snake_case)]
#[inline]
pub const fn RGB(r: u32, g: u32, b: u32) -> u32 {
  r | g << 8 | b << 16
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongW(window, index) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
  unsafe { w32wm::GetWindowLongPtrW(window, index) }
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
  unsafe { w32wm::SetWindowLongW(window, index, value as _) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
  unsafe { w32wm::SetWindowLongPtrW(window, index, value) }
}

/// Implementation of the `GET_X_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_X_LPARAM(lparam: LPARAM) -> i16 {
  ((lparam as usize) & 0xFFFF) as u16 as i16
}

/// Implementation of the `GET_Y_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_Y_LPARAM(lparam: LPARAM) -> i16 {
  (((lparam as usize) & 0xFFFF_0000) >> 16) as u16 as i16
}

pub unsafe fn primary_monitor() -> HMONITOR {
  let pt = POINT { x: 0, y: 0 };
  MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY)
}

pub unsafe fn get_monitor_info(hmonitor: HMONITOR) -> MONITORINFOEXW {
  let mut monitor_info = MONITORINFOEXW {
    szDevice: [0_u16; 32],
    monitorInfo: MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as _,
      dwFlags: 0,
      rcMonitor: RECT {
        bottom: 0,
        left: 0,
        right: 0,
        top: 0,
      },
      rcWork: RECT {
        bottom: 0,
        left: 0,
        right: 0,
        top: 0,
      },
    },
  };
  monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
  GetMonitorInfoW(
    hmonitor,
    &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
  );
  monitor_info
}

struct ComInitialized(*mut ());
impl Drop for ComInitialized {
  fn drop(&mut self) {
    unsafe { CoUninitialize() };
  }
}

thread_local! {
  static COM_INITIALIZED: ComInitialized = {
    unsafe {
        CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED);
        ComInitialized(ptr::null_mut())
    }
  };

  static TASKBAR_LIST: Cell<*mut ITaskbarList> = Cell::new(ptr::null_mut());
}

pub fn com_initialized() {
  COM_INITIALIZED.with(|_| {});
}

pub unsafe fn skip_taskbar(hwnd: HWND) {
  com_initialized();

  TASKBAR_LIST.with(|taskbar_list_ptr| {
    let mut taskbar_list = taskbar_list_ptr.get();

    if taskbar_list.is_null() {
      CoCreateInstance(
        &CLSID_TaskbarList,
        ptr::null_mut(),
        CLSCTX_ALL,
        &IID_ITaskbarList,
        &mut taskbar_list as *mut _ as *mut _,
      );

      let hr_init = (*(*taskbar_list).lpVtbl).HrInit;
      hr_init(taskbar_list.cast());

      taskbar_list_ptr.set(taskbar_list)
    }

    taskbar_list = taskbar_list_ptr.get();
    let delete_tab = (*(*taskbar_list).lpVtbl).DeleteTab;
    delete_tab(taskbar_list, hwnd);
  });
}

pub unsafe fn set_font(hdc: HDC, name: &str, size: i32, weight: i32) {
  let name = format!("{}\0", name);
  let hfont = CreateFontW(
    size,
    0,
    0,
    0,
    weight,
    false.into(),
    false.into(),
    false.into(),
    DEFAULT_CHARSET,
    OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS,
    CLEARTYPE_QUALITY,
    FF_DONTCARE,
    name.as_ptr() as _,
  );
  SelectObject(hdc, hfont);
}

#[allow(dead_code)]
pub(crate) struct Pixel {
  pub(crate) r: u8,
  pub(crate) g: u8,
  pub(crate) b: u8,
  pub(crate) a: u8,
}

impl Pixel {
  fn to_bgra_mut(&mut self) {
    std::mem::swap(&mut self.r, &mut self.b);
  }
}

pub(crate) const PIXEL_SIZE: usize = std::mem::size_of::<Pixel>();

pub fn get_hicon_from_buffer(rgba: Vec<u8>, width: u32, height: u32) -> w32wm::HICON {
  let mut rgba = rgba;
  let pixel_count = rgba.len() / PIXEL_SIZE;
  let mut and_mask = Vec::with_capacity(pixel_count);
  let pixels =
    unsafe { std::slice::from_raw_parts_mut(rgba.as_mut_ptr() as *mut Pixel, pixel_count) };
  for pixel in pixels {
    and_mask.push(pixel.a.wrapping_sub(std::u8::MAX)); // invert alpha channel
    pixel.to_bgra_mut();
  }
  assert_eq!(and_mask.len(), pixel_count);
  unsafe {
    w32wm::CreateIcon(
      HINSTANCE::default(),
      width as i32,
      height as i32,
      1,
      (4 * 8) as u8,
      and_mask.as_ptr() as *const u8,
      rgba.as_ptr() as *const u8,
    ) as w32wm::HICON
  }
}
