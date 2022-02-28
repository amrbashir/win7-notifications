// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;
use std::{
  ptr,
  sync::Mutex,
  thread::{sleep, spawn},
  time::Duration,
};
use windows_sys::Win32::{
  Foundation::*,
  Graphics::{Dwm::*, Gdi::*},
  Media::Audio::*,
  System::LibraryLoader::*,
  UI::{
    Controls::*,
    WindowsAndMessaging::{self as w32wm, *},
  },
};

use crate::{timeout::Timeout, util};

/// notification width
const NW: i32 = 360;
/// notification height
const NH: i32 = 170;
/// notification margin
const NM: i32 = 16;
/// notification icon size (width/height)
const NIS: i32 = 16;
/// notification window bg color
const WC: u32 = util::RGB(50, 57, 69);
/// used for notification summary (title)
const TC: u32 = util::RGB(255, 255, 255);
/// used for notification body
const SC: u32 = util::RGB(200, 200, 200);

static ACTIVE_NOTIFICATIONS: Lazy<Mutex<Vec<HWND>>> = Lazy::new(|| Mutex::new(Vec::new()));
static PRIMARY_MONITOR: Lazy<Mutex<MONITORINFOEXW>> =
  Lazy::new(|| unsafe { Mutex::new(util::get_monitor_info(util::primary_monitor())) });

/// Describes The notification
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Notification {
  pub icon: Vec<u8>,
  pub icon_width: u32,
  pub icon_height: u32,
  pub appname: String,
  pub summary: String,
  pub body: String,
  pub timeout: Timeout,
}

impl Default for Notification {
  fn default() -> Notification {
    Notification {
      appname: util::current_exe_name(),
      summary: String::new(),
      body: String::new(),
      icon: Vec::new(),
      icon_height: 32,
      icon_width: 32,
      timeout: Timeout::Default,
    }
  }
}

impl Notification {
  /// Constructs a new Notification.
  ///
  /// Most fields are empty by default, only `appname` is initialized with the name of the current
  /// executable.
  pub fn new() -> Notification {
    Notification::default()
  }

  /// Overwrite the appname field used for Notification.
  pub fn appname(&mut self, appname: &str) -> &mut Notification {
    self.appname = appname.to_owned();
    self
  }

  /// Set the `summary`.
  ///
  /// Often acts as title of the notification. For more elaborate content use the `body` field.
  pub fn summary(&mut self, summary: &str) -> &mut Notification {
    self.summary = summary.to_owned();
    self
  }

  /// Set the content of the `body` field.
  ///
  /// Multiline textual content of the notification.
  /// Each line should be treated as a paragraph.
  /// html markup is not supported.
  pub fn body(&mut self, body: &str) -> &mut Notification {
    self.body = body.to_owned();
    self
  }

  /// Set the `icon` field from 32bpp RGBA data.
  ///
  /// The length of `rgba` must be divisible by 4, and `width * height` must equal
  /// `rgba.len() / 4`. Otherwise, this will panic.
  pub fn icon(&mut self, rgba: Vec<u8>, width: u32, height: u32) -> &mut Notification {
    if rgba.len() % util::PIXEL_SIZE != 0 {
      // panic!();
    }
    let pixel_count = rgba.len() / util::PIXEL_SIZE;
    if pixel_count != (width * height) as usize {
      // panic!()
    } else {
      self.icon = rgba;
      self.icon_width = width;
      self.icon_height = height;
    }
    self
  }

  /// Set the `timeout` field.
  pub fn timeout(&mut self, timeout: Timeout) -> &mut Notification {
    self.timeout = timeout;
    self
  }

  /// Shows the Notification.
  ///
  /// Requires a win32 event_loop to be running on the thread, otherwise the notification will close immediately.
  pub fn show(&self) -> Result<(), u32> {
    unsafe {
      let hinstance = GetModuleHandleW(ptr::null());

      let class_name = util::encode_wide("win7-notifications");
      let wnd_class = WNDCLASSEXW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: class_name.as_ptr(),
        hInstance: hinstance,
        hbrBackground: CreateSolidBrush(WC),
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: 0,
        hCursor: 0, // must be null in order for cursor state to work properly
        lpszMenuName: ptr::null(),
        hIconSm: 0,
      };
      RegisterClassExW(&wnd_class);

      if let Ok(pm) = PRIMARY_MONITOR.lock() {
        let RECT { right, bottom, .. } = pm.monitorInfo.rcWork;

        let data = WindowData {
          window: 0,
          mouse_hovering_close_btn: false,
          notification: self.clone(),
        };

        let hwnd = CreateWindowExW(
          WS_EX_TOPMOST,
          class_name.as_ptr(),
          util::encode_wide("win7-notifications-window").as_ptr(),
          WS_SYSMENU | WS_CAPTION | WS_VISIBLE,
          right - NW - 15,
          bottom - NH - 15,
          NW,
          NH,
          0,
          0,
          hinstance,
          Box::into_raw(Box::new(data)) as _,
        );

        if hwnd == 0 {
          return Err(GetLastError());
        }

        // re-order active notifications and make room for new one
        if let Ok(mut active_notifications) = ACTIVE_NOTIFICATIONS.lock() {
          active_notifications.push(hwnd);
          let mut i = active_notifications.len() as i32;
          for hwnd in active_notifications.iter() {
            SetWindowPos(
              *hwnd,
              0,
              right - NW - 15,
              bottom - 15 - (NH * i) - 10 * (i - 1),
              0,
              0,
              SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
            );
            i -= 1;
          }
        }

        // shadows
        let mut is_dwm_enabled = 0;
        DwmIsCompositionEnabled(&mut is_dwm_enabled);
        if is_dwm_enabled == 1 {
          let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 0,
            cyBottomHeight: 0,
            cyTopHeight: 0,
          };
          DwmExtendFrameIntoClientArea(hwnd, &margins);
        }

        util::skip_taskbar(hwnd);
        ShowWindow(hwnd, SW_SHOW);
        // Passing an invalid path to `PlaySoundW` will make windows play default sound.
        // https://docs.microsoft.com/en-us/previous-versions/dd743680(v=vs.85)#remarks
        PlaySoundW(util::encode_wide("null").as_ptr(), hinstance, SND_ASYNC);

        let timeout = self.timeout;
        spawn(move || {
          sleep(Duration::from_millis(timeout.into()));
          if timeout != Timeout::Never {
            close_notification(hwnd);
          };
        });
      }
    }

    Ok(())
  }
}

unsafe fn close_notification(hwnd: HWND) {
  ShowWindow(hwnd, SW_HIDE);
  CloseWindow(hwnd);

  if let Ok(mut active_noti) = ACTIVE_NOTIFICATIONS.lock() {
    if let Some(index) = active_noti.iter().position(|e| *e == hwnd) {
      active_noti.remove(index);
    }

    // re-order notifications
    if let Ok(pm) = PRIMARY_MONITOR.lock() {
      let RECT { right, bottom, .. } = pm.monitorInfo.rcWork;
      for (i, h) in active_noti.iter().rev().enumerate() {
        SetWindowPos(
          *h,
          0,
          right - NW - 15,
          bottom - (NH * (i + 1) as i32) - 15,
          0,
          0,
          SWP_NOSIZE | SWP_NOZORDER,
        );
      }
    }
  }
}

struct WindowData {
  window: HWND,
  notification: Notification,
  mouse_hovering_close_btn: bool,
}

pub unsafe extern "system" fn window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let mut userdata = util::GetWindowLongPtrW(hwnd, GWL_USERDATA);

  match msg {
    w32wm::WM_NCCREATE => {
      if userdata == 0 {
        let createstruct = &*(lparam as *const CREATESTRUCTW);
        userdata = createstruct.lpCreateParams as isize;
        util::SetWindowLongPtrW(hwnd, GWL_USERDATA, userdata);
      }
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    // make the window borderless
    w32wm::WM_NCCALCSIZE => 0,

    w32wm::WM_CREATE => {
      let userdata = userdata as *mut WindowData;
      (*userdata).window = hwnd;
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_PAINT => {
      let userdata = userdata as *mut WindowData;
      let mut ps = PAINTSTRUCT {
        fErase: 0,
        fIncUpdate: 0,
        fRestore: 0,
        hdc: 0,
        rcPaint: RECT {
          bottom: 0,
          left: 0,
          right: 0,
          top: 0,
        },
        rgbReserved: [0 as u8; 32],
      };
      let hdc = BeginPaint(hwnd, &mut ps);

      SetBkColor(hdc, WC);
      SetTextColor(hdc, TC);

      // draw notification icon
      let hicon = util::get_hicon_from_buffer(
        (*userdata).notification.icon.clone(),
        (*userdata).notification.icon_width,
        (*userdata).notification.icon_height,
      );
      DrawIconEx(hdc, NM, NM, hicon, NIS, NIS, 0, 0, DI_NORMAL);

      // draw notification close button
      util::set_font(hdc, "", 16, 700);
      SetTextColor(
        hdc,
        if (*userdata).mouse_hovering_close_btn {
          TC
        } else {
          SC
        },
      );
      TextOutW(
        hdc,
        NW - NM - NM / 2,
        NM,
        util::encode_wide("X").as_ptr(),
        1,
      );

      // draw notification app name
      SetTextColor(hdc, TC);
      util::set_font(hdc, "Segeo UI", 15, 400);
      let appname = util::encode_wide((*userdata).notification.appname.clone());
      TextOutW(
        hdc,
        NM + NIS + (NM / 2),
        NM,
        appname.as_ptr(),
        appname.len() as _,
      );

      // draw notification summary (title)
      util::set_font(hdc, "Segeo UI", 17, 700);
      let summary = util::encode_wide((*userdata).notification.summary.clone());
      TextOutW(
        hdc,
        NM,
        NM + NIS + (NM / 2),
        summary.as_ptr(),
        summary.len() as _,
      );

      // draw notification body
      SetTextColor(hdc, SC);
      util::set_font(hdc, "Segeo UI", 17, 400);
      let mut rc = RECT {
        left: NM,
        top: NM + NIS + (NM / 2) + 17 + (NM / 2),
        right: NW - NM,
        bottom: NH - NM,
      };
      let body = util::encode_wide((*userdata).notification.body.clone());
      DrawTextW(
        hdc,
        body.as_ptr(),
        body.len() as _,
        &mut rc,
        DT_LEFT | DT_EXTERNALLEADING | DT_WORDBREAK,
      );

      EndPaint(hdc, &ps);
      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_MOUSEMOVE => {
      let userdata = userdata as *mut WindowData;

      let (x, y) = (util::GET_X_LPARAM(lparam), util::GET_Y_LPARAM(lparam));
      let hit = close_button_hit_test(x, y);
      SetCursor(LoadCursorW(0, if hit { IDC_HAND } else { IDC_ARROW }));
      if hit != (*userdata).mouse_hovering_close_btn {
        // only trigger redraw if the previous state is different than the new state
        InvalidateRect(hwnd, std::ptr::null(), 0);
      }
      (*userdata).mouse_hovering_close_btn = hit;

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_LBUTTONDOWN => {
      let (x, y) = (util::GET_X_LPARAM(lparam), util::GET_Y_LPARAM(lparam));
      if close_button_hit_test(x, y) {
        close_notification(hwnd)
      }

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    w32wm::WM_DESTROY => {
      let userdata = userdata as *mut WindowData;
      Box::from_raw(userdata);

      DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

fn close_button_hit_test(x: i16, y: i16) -> bool {
  (x > (NW - NM - NM) as i16)
    && (x < (NW - NM / 2) as i16)
    && (y > NM as i16)
    && (y < (NM * 2) as i16)
}
