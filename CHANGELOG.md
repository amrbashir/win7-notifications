# Changelog

## \[0.4.5]

- [`e9d5b36`](https://github.com/tauri-apps/win7-notifications/commit/e9d5b36fcca1824ffa7971e7456eb142876b1d71) Update `windows-sys` crate to `0.59`

## \[0.4.4]

- [`a0fbb67`](https://github.com/tauri-apps/win7-notifications/commit/a0fbb67ed22056ba3cd61842a3e5abd023f0185c) ([#65](https://github.com/tauri-apps/win7-notifications/pull/65)) Fix notification window causes the main window to lose focus.

## \[0.4.3]

- [`c28f8f6`](https://github.com/tauri-apps/win7-notifications/commit/c28f8f61a0c1c906bacc9b5b740d8ea343707afb) Update `windows-sys` crate to `0.52`

## \[0.4.2]

- [`13e2cbc`](https://github.com/tauri-apps/win7-notifications/commit/13e2cbcdcb59a6dc9c3a6588b9e4f57ac6662fbf)([#55](https://github.com/tauri-apps/win7-notifications/pull/55)) Relax version requirement of `once_cell` crate to `1`.

## \[0.4.1]

- [`62bbd8b`](https://github.com/tauri-apps/win7-notifications/commit/62bbd8b3ed55467b76c16fadb843e060804ea2fe) Fix notifications were minimized but not actually closed by sending `WM_CLOSE`.

## \[0.4.0]

- [`c3c0136`](https://github.com/tauri-apps/win7-notifications/commit/c3c013691eeb71693ed2aa5c6f8b856e6c5938f4)([#50](https://github.com/tauri-apps/win7-notifications/pull/50)) Add option to make the notification silent

## \[0.3.1]

- Fix crash when a notification doesn't have an icon.
  - [eabcae6](https://github.com/tauri-apps/win7-notifications/commit/eabcae6edb0443cb1ca41ff45815d7de9002d0a3) fix: fix crash when a notficiation didn't have an icon on 2022-06-28

## \[0.3.0]

- Properly clean notifications data from memory.
  - [4a1e746](https://github.com/tauri-apps/win7-notifications/commit/4a1e7465fa5623d48dcd74e57e937fa2ae471ab8) fix: clean notification data when it is closed on 2022-01-01
- Change close button color when mouse is hovering.
  - [80bca00](https://github.com/tauri-apps/win7-notifications/commit/80bca0085d3395e9c902613a879c0c5242f5ff0c) feat: change close btn color when hovering on 2022-01-05
- Fix cursor style when the mouse is over the notification.
  - [2f025e8](https://github.com/tauri-apps/win7-notifications/commit/2f025e8f585ba7458cc3e756af13d1f6f6908864) fix: fix cursor style is stuck on loading on 2022-01-05
- Add gaps between multiple notifications
  - [a47bc6b](https://github.com/tauri-apps/win7-notifications/commit/a47bc6b8315b03f55d1d8f104500d4bea65360d1) feat: add gaps between notifications on 2022-01-06
- **Breaking change** Notification icon must be "32bpp RGBA data" and width and height must be specified now, check `Notification::icon` for more info.
  - [a52b763](https://github.com/tauri-apps/win7-notifications/commit/a52b76383fd41497464f8b71ca10551f0202ca55) feat: improve icon handling on 2022-02-28
- Migrate to `windows-sys` crate.
  - [8bf78a2](https://github.com/tauri-apps/win7-notifications/commit/8bf78a215a500e6e6018f7a31cfc78d8c7e588c3) feat: migrate to `windows-sys` crate on 2022-02-28
  - [8888d21](https://github.com/tauri-apps/win7-notifications/commit/8888d21be39c430b1d2d3285ea02569586a905cc) chore: fix covector publish on 2022-03-05

## \[0.2.3]

- Migrate to rust edition 2021.
  - [5d77b46](https://github.com/tauri-apps/win7-notifications/commit/5d77b46fe7f45b752015537c839a0feae76717f1) docs: update documentation on 2021-10-31

## \[0.2.2]

- Check if DWM is enabled or not before adding shadows.
  - [84a9556](https://github.com/tauri-apps/win7-notifications/commit/84a9556aaa239caead8b7111796047a438845be9) fix: check if dwm is enabled before adding shadows on 2021-10-05
- Fix crash when clicking close button.
  - [1487210](https://github.com/tauri-apps/win7-notifications/commit/14872100c78f6ddda2ee9b3a660bdf1b186b2ce3) fix: fix crash on click close button on 2021-10-05

## \[0.2.1]

- Fix first notification not showing.
  - [78533c5](https://github.com/tauri-apps/win7-notifications/commit/78533c59ca880a699d4d312f03fe635b6f287371) fix: first notification not showing on 2021-10-05

## \[0.2.0]

- Move old notifications above new ones
  - [2fdc444](https://github.com/tauri-apps/win7-notifications/commit/2fdc4442f593334aee513dbfe2bffbb29aef5fe0) feat: move old notifications above new one on 2021-10-05
- Add shadows
  - [c1c7dd2](https://github.com/tauri-apps/win7-notifications/commit/c1c7dd27949ba34a0395061f64a912aaa47c9c2e) feat: add shadows on 2021-10-05
- Add sounds
  - [f667d6f](https://github.com/tauri-apps/win7-notifications/commit/f667d6fce3d52ee49e0c1af03b1507383ab67eab) feat: add sounds on 2021-10-05

## \[0.1.0]

- Initial Release.
  - [2a0990b](https://github.com/tauri-apps/win7-notifications/commit/2a0990bcc750178a24e38cf0293c2944c01596dc) add `initial-release.md` on 2021-10-04
