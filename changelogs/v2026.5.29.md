# v2026.5.29

## Alert Countdowns

- Both encounter timers and effects can now render countdown alerts. These can be configured in the alerts section of the
  editor card. They show a ticking countdown in the format of: **Alert Text (2.1)**

## Overlays

- Timers, Boss HP, and bar-mode effects/cooldown overlays now have customizable borders around their outlines. Border
  color can be set in the customization menu.
- Boss HP overlay format has been updated to be more compact and readable.
- Overlays can now be scaled to larger sizes
- The maximum value of the raid frame spacing has been increased to 75px

## Audio

- Over 300 audio files containing mechanic names generated via TTS by Keetsune have been added
- A filter option has been added to the UI

## Other

- Max Hit and average per-activation columns have been added to the data explorer
- Updated slider widgets in customization menu to support typing in values

## Bugfixes

- Shielding is now properly attributed to it's source in the HTPS data explorer view
- Effective damage overlay now records damage done to shields
- Metrics overlay footer totals now scale properly with the entry text
- Red Acid Jet timer should now refresh if ability is cast slightly before timer expires
