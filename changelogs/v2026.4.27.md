# v2026.4.27

## v2026.4.28 Hotfix

- Fixed edge case where huntmaster classification could be wrong
- Readded Master Blaster Victory trigger

## macOS Support

All features should now be working on macOS

- Overlays now show over the game window
- Application icon now shows in system tray
- Installing the application now displays a message saying it's unsigned, rather than damaged

## Linux Support

- Updated monitor ID on Linux Wayland to always include connector port

## Overlays

- Switched over overlay font to bundled `Inter` font
- All text is now rendered with text glow for enhanced readability
- Improved formatting of discipline icons on metrics overlays
- Updated Text/icon formatting of challenges overlay to match regular metrics overlays
- Reduced the amount of spacing between text and HP bar on HP overlay
- Major improvements to boss ability queue overlay, though feature is still in beta
- Entities with shields will always appear on HP overlay, even if HP bar is full
- Added option to display timers on multiple overlays simultaneously

## Encounters

- Definition files have been added for all remaining flashpoints (thanks Sinrai)
- Dxun II and III are now detected immediately on encounter start
- NAHUT is now detected when a player picks up the candle
- Olok encounter is now detected on Wealthy Buyer
- Added EV Pylons encounter (lol)
- Corruptor Zero gravity timer should now be accurate
- Operator IX phase timers should now be accurate without requiring the player to direct cast on the hologram
- Added Styrak timers
- Added Master and Blaster Timers
- Various updates to TFB and DF timers

## Parsely

- Parsely upload now supports multiple guilds

## Bugfixes

- Operation timer is no longer reset upon re-entering an operation area
- When multiple shields of the same name are active, the application now checks for the proper entity before removing them
- Various rendering performance fixes and optimizations
