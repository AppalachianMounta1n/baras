# v2026.6.17

## Improved Combat Log Filtering

Combat log now supports the AND keyword as well as composition within parenthesis. Example: `(Dark Ward AND Player_Name) AND NOT Dark Bulwark`

## Effect Modifiers

Implemented optional modifiers from tracked effects/cooldowns. These are meant to model non-standard behavior such as thermal yield refreshing on damage, cooldowns being reduced by events etc. This feature is experimental.

## Other Features

- Added APM Overlay
- Improved chart tooltip formatting
- Added option for AOE refresh to always fire on damage (for things that aren't corrosive grenade basically)
- Overlay settings panel is now free floating; allowing for interaction with overlay controls while the menu is open
- Alacrity value can now be set to 2 decimal places

## Bugfixes

- Fixed name collision error in effects selection on charts tab
- Huntmaster encounter should now be guaranteed to end upon entering a cutscene (opening Apex door) preventing the parser from freezing
- Updated Dxun encounter victory conditions to handle more edge cases
- Creeping terror and Sever Force tracked DOTs are set to filer target to "Any Except Local" by default
- All dots have been set to track outside of combat = false by default

## Definitions

- Various HP markers and HP pushes_at values have been added to encounters
