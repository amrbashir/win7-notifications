// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;
use std::{ptr, sync::Mutex, thread, time::Duration};
use windows_sys::{
    w,
    Win32::{
        Foundation::*, Graphics::Gdi::*, Media::Audio::*, System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
    },
};

use crate::{
    timeout::Timeout,
    util::{self, GetWindowLongPtrW, SetWindowLongPtrW, GET_X_LPARAM, GET_Y_LPARAM, RGB},
};

/// notification width
const NW: i32 = 360;
/// notification height
const NH: i32 = 170;
/// notification margin
const NM: i32 = 16;
/// notification icon size (width or height)
const NIS: i32 = 16;
/// notification window bg color
const WC: u32 = RGB(50, 57, 69);
/// used for notification summary (title)
const TC: u32 = RGB(255, 255, 255);
/// used for notification body
const SC: u32 = RGB(200, 200, 200);

const CLOSE_BTN_RECT: RECT = RECT {
    left: NW - NM - NM / 2,
    top: NM,
    right: (NW - NM - NM / 2) + 8,
    bottom: NM + 8,
};
const CLOSE_BTN_RECT_EXTRA: RECT = RECT {
    left: CLOSE_BTN_RECT.left - 8,
    top: CLOSE_BTN_RECT.top - 8,
    right: CLOSE_BTN_RECT.right + 8,
    bottom: CLOSE_BTN_RECT.bottom + 8,
};

static CURRENT_WINDOW: Lazy<Mutex<Option<HWND>>> = Lazy::new(|| Mutex::new(None));
static PRIMARY_MONITOR: Lazy<Mutex<MONITORINFOEXW>> =
    Lazy::new(|| unsafe { Mutex::new(util::get_monitor_info(util::primary_monitor())) });

/// Describes The notification
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Notification {
    pub icon: Option<Vec<u8>>,
    pub icon_width: u32,
    pub icon_height: u32,
    pub appname: String,
    pub summary: String,
    pub body: String,
    pub timeout: Timeout,
    pub silent: bool,
}

impl Default for Notification {
    fn default() -> Notification {
        Notification {
            appname: util::current_exe_name(),
            summary: String::new(),
            body: String::new(),
            icon: None,
            icon_height: 32,
            icon_width: 32,
            timeout: Timeout::Default,
            silent: false,
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
            panic!("The length of `rgba` is not divisible by 4");
        }
        let pixel_count = rgba.len() / util::PIXEL_SIZE;
        if pixel_count != (width * height) as usize {
            panic!("`width * height` is not equal `rgba.len() / 4`");
        } else {
            self.icon = Some(rgba);
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

    /// Set the `silent` field.
    pub fn silent(&mut self, silent: bool) -> &mut Notification {
        self.silent = silent;
        self
    }

    /// Shows the Notification.
    ///
    /// Requires a win32 event_loop to be running on the thread, otherwise the notification will close immediately.
    pub fn show(&self) -> Result<(), u32> {
        unsafe {
            // close prev window
            if let Ok(mut current_window) = CURRENT_WINDOW.lock() {
                if let Some(hwnd) = *current_window {
                    close_notification(hwnd);
                }
            }
            let hinstance = GetModuleHandleW(ptr::null());

            let class_name = w!("win7-notifications");
            let wnd_class = WNDCLASSEXW {
                lpfnWndProc: Some(window_proc),
                lpszClassName: class_name,
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
                    WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                    class_name,
                    w!("win7-notifications-window"),
                    WS_POPUP | WS_BORDER,
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

                // Stores the current hwnd window
                if let Ok(mut current_window) = CURRENT_WINDOW.lock() {
                    *current_window = Some(hwnd);
                }

                SetWindowPos(
                    hwnd,
                    0,
                    right - NW - 15,
                    bottom - NH - 15,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
                );

                ShowWindow(hwnd, SW_SHOWNA);
                if !self.silent {
                    // Passing an invalid path to `PlaySoundW` will make windows play default sound.
                    // https://docs.microsoft.com/en-us/previous-versions/dd743680(v=vs.85)#remarks
                    PlaySoundW(w!("null"), hinstance, SND_ASYNC);
                }

                let timeout = self.timeout;
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(timeout.into()));
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

    // We can NOT call `DestroyWindow` from this window
    // Sending WM_CLOSE will by default make the windows call it on itself.
    // Note WM_DESTROY should not be sent directly as it would create a leak
    // see https://devblogs.microsoft.com/oldnewthing/20110926-00/?p=9553
    SendMessageA(hwnd, WM_CLOSE, 0, 0);
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
    let mut userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);

    match msg {
        WM_NCCREATE => {
            if userdata == 0 {
                let createstruct = &*(lparam as *const CREATESTRUCTW);
                userdata = createstruct.lpCreateParams as isize;
                SetWindowLongPtrW(hwnd, GWL_USERDATA, userdata);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_CREATE => {
            let userdata = userdata as *mut WindowData;
            (*userdata).window = hwnd;
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_PAINT | WM_PRINTCLIENT => {
            let userdata = userdata as *mut WindowData;
            let notification = &(*userdata).notification;

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
                rgbReserved: [0; 32],
            };
            let hdc = BeginPaint(hwnd, &mut ps);
            SetBkColor(hdc, WC);

            // draw notification icon
            if let Some(icon) = &notification.icon {
                let hicon = util::get_hicon_from_32bpp_rgba(
                    icon.clone(),
                    notification.icon_width,
                    notification.icon_height,
                );
                DrawIconEx(hdc, NM, NM, hicon, NIS, NIS, 0, 0, DI_NORMAL);
            }

            // draw notification close button
            let hpen = CreatePen(
                PS_SOLID,
                2,
                if (*userdata).mouse_hovering_close_btn {
                    TC
                } else {
                    SC
                },
            );
            let old_hpen = SelectObject(hdc, hpen);
            MoveToEx(
                hdc,
                CLOSE_BTN_RECT.left,
                CLOSE_BTN_RECT.top,
                std::ptr::null_mut(),
            );
            LineTo(hdc, CLOSE_BTN_RECT.right, CLOSE_BTN_RECT.bottom);
            MoveToEx(
                hdc,
                CLOSE_BTN_RECT.right,
                CLOSE_BTN_RECT.top,
                std::ptr::null_mut(),
            );
            LineTo(hdc, CLOSE_BTN_RECT.left, CLOSE_BTN_RECT.bottom);
            SelectObject(hdc, old_hpen);
            DeleteObject(hpen);

            // draw notification app name
            SetTextColor(hdc, TC);
            let (hfont, old_hfont) = util::set_font(hdc, "Segeo UI", 15, 400);
            let appname = util::encode_wide(&notification.appname);
            TextOutW(
                hdc,
                NM + NIS + (NM / 2),
                NM,
                appname.as_ptr(),
                appname.len() as _,
            );
            SelectObject(hdc, old_hfont);
            DeleteObject(hfont);

            // draw notification summary (title)
            let (hfont, old_hfont) = util::set_font(hdc, "Segeo UI", 17, 700);
            let summary = util::encode_wide(&notification.summary);
            TextOutW(
                hdc,
                NM,
                NM + NIS + (NM / 2),
                summary.as_ptr(),
                summary.len() as _,
            );
            SelectObject(hdc, old_hfont);
            DeleteObject(hfont);

            // draw notification body
            SetTextColor(hdc, SC);
            let (hfont, old_hfont) = util::set_font(hdc, "Segeo UI", 17, 400);
            let mut rc = RECT {
                left: NM,
                top: NM + NIS + (NM / 2) + 17 + (NM / 2),
                right: NW - NM,
                bottom: NH - NM,
            };
            let mut body = util::encode_wide(&notification.body);
            DrawTextW(
                hdc,
                body.as_mut_ptr(),
                body.len() as _,
                &mut rc,
                DT_LEFT | DT_EXTERNALLEADING | DT_WORDBREAK,
            );
            SelectObject(hdc, old_hfont);
            DeleteObject(hfont);

            EndPaint(hdc, &ps);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_MOUSEMOVE => {
            let userdata = userdata as *mut WindowData;

            let (x, y) = (GET_X_LPARAM(lparam), GET_Y_LPARAM(lparam));
            let hit = util::rect_contains(CLOSE_BTN_RECT_EXTRA, x as i32, y as i32);

            SetCursor(LoadCursorW(0, if hit { IDC_HAND } else { IDC_ARROW }));
            if hit != (*userdata).mouse_hovering_close_btn {
                // only trigger redraw if the previous state is different than the new state
                InvalidateRect(hwnd, std::ptr::null(), 0);
            }
            (*userdata).mouse_hovering_close_btn = hit;

            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_LBUTTONDOWN => {
            let (x, y) = (GET_X_LPARAM(lparam), GET_Y_LPARAM(lparam));

            if util::rect_contains(CLOSE_BTN_RECT_EXTRA, x as i32, y as i32) {
                close_notification(hwnd)
            }

            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_DESTROY => {
            let userdata = userdata as *mut WindowData;
            drop(Box::from_raw(userdata));

            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
