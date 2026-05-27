//! Boss Health Bar Overlay
//!
//! Displays real-time health bars for boss NPCs in the current encounter.
//! Supports HP threshold markers (vertical lines at key HP%) and shield bars.

use std::collections::HashMap;
use std::sync::Arc;

use baras_core::context::BossHealthConfig;
use baras_core::OverlayHealthEntry;
use tiny_skia::Color;

use super::{Overlay, OverlayConfigUpdate, OverlayData};
use crate::frame::OverlayFrame;
use crate::platform::{OverlayConfig, PlatformError};
use crate::utils::color_from_rgba;
use crate::widgets::colors;
use crate::widgets::ProgressBar;
use baras_types::formatting;

/// A single effect icon to render beneath a boss HP bar.
#[derive(Debug, Clone)]
pub struct BossEffectIcon {
    pub effect_id: u64,
    pub icon_ability_id: u64,
    pub name: String,
    pub remaining_secs: f32,
    pub total_secs: f32,
    pub color: [u8; 4],
    pub show_icon: bool,
    pub icon: Option<Arc<(u32, u32, Vec<u8>)>>,
}

impl BossEffectIcon {
    pub fn progress(&self) -> f32 {
        if self.total_secs > 0.0 {
            (self.remaining_secs / self.total_secs).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    pub fn format_time(&self, european: bool) -> String {
        formatting::format_countdown_compact(self.remaining_secs, "0", european)
    }
}

/// Data sent from service to boss health overlay
#[derive(Debug, Clone, Default)]
pub struct BossHealthData {
    /// Current boss health entries (sorted by encounter order)
    pub entries: Vec<OverlayHealthEntry>,
    /// Effect icons keyed by NPC entity id (matches OverlayHealthEntry::entity_id).
    /// Keyed by id rather than name so two NPCs that share a display name show
    /// only the effects actually applied to each one.
    pub boss_icons: HashMap<i64, Vec<BossEffectIcon>>,
}

/// Base dimensions for scaling calculations
const BASE_WIDTH: f32 = 250.0;
const BASE_HEIGHT: f32 = 100.0;

/// Base layout values (at BASE_WIDTH x BASE_HEIGHT)
const BASE_BAR_HEIGHT: f32 = 20.0;
const BASE_ENTRY_SPACING: f32 = 8.0;
const BASE_LABEL_BAR_GAP: f32 = 1.0;
const BASE_PADDING: f32 = 8.0;
const BASE_FONT_SIZE: f32 = 13.0;
const BASE_LABEL_FONT_SIZE: f32 = 8.5;

/// Shield bar height (thinner than HP bar)
const BASE_SHIELD_BAR_HEIGHT: f32 = 12.0;

fn shield_bar_color() -> Color {
    Color::from_rgba8(100, 180, 255, 200)
}

/// Replace each ASCII digit with '8' (typically the widest digit in
/// proportional fonts). Used to size column reservations so the actual
/// rendered text never exceeds the reserved width, regardless of which
/// digits appear.
fn digit_normalize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_digit() { '8' } else { c })
        .collect()
}

fn marker_line_color() -> Color {
    Color::from_rgba8(255, 255, 255, 180)
}

/// Maximum number of bosses we optimize scaling for
const MAX_SUPPORTED_BOSSES: usize = 7;
/// Minimum compression factor to keep entries readable
const MIN_COMPRESSION: f32 = 0.4;

/// Boss health bar overlay
pub struct BossHealthOverlay {
    frame: OverlayFrame,
    config: BossHealthConfig,
    data: BossHealthData,
    european_number_format: bool,
    /// (current, max, shield_remaining_per_shield) per entry — used to skip re-renders
    /// when HP and shields are unchanged and no boss effects are ticking. Tracking
    /// per-shield `remaining` (not just count) lets the shield bar animate as it absorbs.
    last_hp_sig: Vec<(i32, i32, Vec<i64>)>,
    /// Total boss-effect icon count from the last frame. Forces one final re-render
    /// on the trailing edge when icons disappear, so a stale "0.0" countdown text
    /// doesn't remain on screen.
    last_icon_count: usize,
}

impl BossHealthOverlay {
    /// Create a new boss health overlay
    pub fn new(
        window_config: OverlayConfig,
        config: BossHealthConfig,
        background_alpha: u8,
    ) -> Result<Self, PlatformError> {
        let mut frame = OverlayFrame::new(window_config, BASE_WIDTH, BASE_HEIGHT)?;
        frame.set_background_alpha(background_alpha);
        frame.set_label("Boss Health");

        Ok(Self {
            frame,
            config,
            data: BossHealthData::default(),
            european_number_format: false,
            last_hp_sig: Vec::new(),
            last_icon_count: 0,
        })
    }

    /// Update the config
    pub fn set_config(&mut self, config: BossHealthConfig) {
        self.config = config;
    }

    /// Update background alpha
    pub fn set_background_alpha(&mut self, alpha: u8) {
        self.frame.set_background_alpha(alpha);
    }

    /// Update the data
    pub fn set_data(&mut self, data: BossHealthData) {
        self.data = data;
    }

    /// Calculate scaled font size so text fits within max_width
    fn scaled_font_for_text(&mut self, text: &str, max_width: f32, base_font_size: f32) -> f32 {
        let (text_width, _) = self.frame.measure_text(text, base_font_size);
        if text_width <= max_width {
            return base_font_size;
        }

        // Scale font proportionally to fit
        let scale = max_width / text_width;
        let min_font = base_font_size * 0.6; // Don't go below 60% of base size
        (base_font_size * scale).max(min_font)
    }

    /// Calculate per-entry height for a given entry (accounts for shields and icon row).
    ///
    /// `icon_row_height`: `Some(h)` when an icon strip is rendered below the bar
    /// (overrides the plain marker-text row). `None` falls back to marker-text-only.
    fn entry_height(
        &self,
        entry: &OverlayHealthEntry,
        bar_height: f32,
        label_bar_gap: f32,
        label_font_size: f32,
        shield_bar_height: f32,
        icon_row_height: Option<f32>,
    ) -> f32 {
        // Compact inline layout: name + HP + percent live on the HP bar
        // itself. Target renders as a "badge" half-overlapping the bar's
        // bottom-right corner — its lower half extends below the bar.
        let mut h = 0.0;

        // Shield bar (above the HP bar, when present)
        if !entry.active_shields.is_empty() {
            h += shield_bar_height + label_bar_gap;
        }

        h += bar_height;

        // Target badge floats over the layout — does not contribute to
        // height. It overlaps the icons/marker row visually if present.

        if let Some(row_h) = icon_row_height {
            h += row_h;
        } else if Self::next_marker(entry).is_some() {
            h += label_font_size * 0.85 + 2.0;
        }

        h
    }

    /// Icon row height for a given bar height (3px gap above icons + icon size + 3px gap below).
    fn icon_row_height(bar_height: f32) -> f32 {
        bar_height * 0.6 + 6.0
    }

    /// Calculate compression factor to fit entries in available height
    fn compression_factor(&self, entries: &[OverlayHealthEntry]) -> f32 {
        let height = self.frame.height() as f32;
        let font_scale = self.config.font_scale.clamp(0.3, 2.0);
        let padding = self.frame.scaled(BASE_PADDING);
        let bar_height = self.frame.scaled(BASE_BAR_HEIGHT);
        let entry_spacing = self.frame.scaled(BASE_ENTRY_SPACING);
        let label_bar_gap = self.frame.scaled(BASE_LABEL_BAR_GAP);
        let label_font_size = self.frame.scaled(BASE_LABEL_FONT_SIZE) * font_scale;
        let shield_bar_height = self.frame.scaled(BASE_SHIELD_BAR_HEIGHT);

        let icon_row_h = Self::icon_row_height(bar_height);

        let total_needed: f32 = padding * 2.0
            + entries
                .iter()
                .map(|e| {
                    let has_icons = self.data.boss_icons.get(&e.entity_id).is_some_and(|v| !v.is_empty());
                    let icon_row = (has_icons || Self::next_marker(e).is_some()).then_some(icon_row_h);
                    self.entry_height(
                        e,
                        bar_height,
                        label_bar_gap,
                        label_font_size,
                        shield_bar_height,
                        icon_row,
                    ) + entry_spacing
                })
                .sum::<f32>()
            - entry_spacing;

        if total_needed <= height {
            1.0
        } else {
            (height / total_needed).max(MIN_COMPRESSION)
        }
    }

    /// Pre-compute the total content height for all visible entries.
    fn compute_content_height(&self, entries: &[OverlayHealthEntry], compression: f32) -> f32 {
        let font_scale = self.config.font_scale.clamp(0.3, 2.0);
        let padding = self.frame.scaled(BASE_PADDING);
        let bar_height = self.frame.scaled(BASE_BAR_HEIGHT) * compression;
        let entry_spacing = self.frame.scaled(BASE_ENTRY_SPACING) * compression;
        let label_bar_gap = self.frame.scaled(BASE_LABEL_BAR_GAP) * compression;
        let label_font_size = self.frame.scaled(BASE_LABEL_FONT_SIZE) * compression * font_scale;
        let shield_bar_height = self.frame.scaled(BASE_SHIELD_BAR_HEIGHT) * compression;

        let icon_row_h = Self::icon_row_height(bar_height);
        let mut y = padding;

        for entry in entries {
            let has_icons = self.data.boss_icons.get(&entry.entity_id).is_some_and(|v| !v.is_empty());
            let icon_row = (has_icons || Self::next_marker(entry).is_some()).then_some(icon_row_h);
            y += self.entry_height(
                entry,
                bar_height,
                label_bar_gap,
                label_font_size,
                shield_bar_height,
                icon_row,
            );
            y += entry_spacing;
        }

        // Replace the trailing entry_spacing with bottom padding
        if !entries.is_empty() {
            y = y - entry_spacing + padding;
        }

        y
    }

    /// Find the next relevant HP marker: the highest hp_percent that is <= current HP%.
    /// This is the next threshold the boss will cross as HP decreases.
    fn next_marker(entry: &OverlayHealthEntry) -> Option<(f32, &str)> {
        let current_pct = entry.percent();
        entry
            .hp_markers
            .iter()
            .filter(|m| m.hp_percent <= current_pct)
            .max_by(|a, b| {
                a.hp_percent
                    .partial_cmp(&b.hp_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|m| (m.hp_percent, m.label.as_str()))
    }

    /// Render a skeleton preview when in move mode (1 boss)
    fn render_preview(&mut self) {
        let width = self.frame.width() as f32;

        let font_scale = self.config.font_scale.clamp(0.3, 2.0);
        let padding = self.frame.scaled(BASE_PADDING);
        let bar_height = self.frame.scaled(BASE_BAR_HEIGHT);
        let font_size = self.frame.scaled(BASE_FONT_SIZE) * font_scale;
        let bar_radius = 4.0 * self.frame.scale_factor();

        let bar_color = color_from_rgba(self.config.bar_color);
        let font_color = color_from_rgba(self.config.font_color);

        let content_width = width - padding * 2.0;

        self.frame.begin_frame();

        let y = padding;

        let name = "Boss";
        let health_text = "1.2M";
        let percent_text = if self.config.show_percent {
            "72.0%".to_string()
        } else {
            String::new()
        };
        let bar_font_size = font_size * 0.70;

        // Bar background + fill (no inline text — three columns layered next).
        ProgressBar::new("", 0.72)
            .with_fill_color(bar_color)
            .with_bg_color(colors::dps_bar_bg())
            .render(
                &mut self.frame,
                padding,
                y,
                content_width,
                bar_height,
                bar_font_size,
                bar_radius,
            );

        if self.config.show_border {
            self.frame.stroke_rounded_rect(
                padding,
                y,
                content_width,
                bar_height,
                bar_radius,
                0.8 * self.frame.scale_factor(),
                color_from_rgba(self.config.border_color),
            );
        }

        // Three-column inline text matching the live render path.
        let inline_text_pad = 6.0 * self.frame.scale_factor();
        let inline_col_gap = 8.0 * self.frame.scale_factor();
        let (hp_col_w, _) = self.frame.measure_text("123.45M", bar_font_size);
        let (pct_col_w, _) = if self.config.show_percent {
            self.frame.measure_text("12.3%", bar_font_size)
        } else {
            (0.0, 0.0)
        };
        let pct_section = if self.config.show_percent {
            inline_col_gap + pct_col_w
        } else {
            0.0
        };
        let name_col_w = (content_width
            - inline_text_pad * 2.0
            - inline_col_gap
            - hp_col_w
            - pct_section)
            .max(0.0);

        let name_font_size = self.scaled_font_for_text(name, name_col_w, bar_font_size);
        let text_y = y + bar_height / 2.0 + bar_font_size / 3.0;
        self.frame.draw_text_glowed(
            name,
            padding + inline_text_pad,
            text_y,
            name_font_size,
            font_color,
        );

        let hp_col_x = padding + inline_text_pad + name_col_w + inline_col_gap;
        let (hp_w, _) = self.frame.measure_text(health_text, bar_font_size);
        self.frame.draw_text_glowed(
            health_text,
            hp_col_x + hp_col_w - hp_w,
            text_y,
            bar_font_size,
            font_color,
        );

        if self.config.show_percent {
            let pct_col_x = hp_col_x + hp_col_w + inline_col_gap;
            let (pct_w, _) = self.frame.measure_text(&percent_text, bar_font_size);
            self.frame.draw_text_glowed(
                &percent_text,
                pct_col_x + pct_col_w - pct_w,
                text_y,
                bar_font_size,
                font_color,
            );
        }

        self.frame.end_frame();
    }

    /// Render the overlay
    pub fn render(&mut self) {
        if self.frame.is_in_move_mode() {
            self.render_preview();
            return;
        }

        let width = self.frame.width() as f32;

        // Filter out dead bosses (0% health) and pushed bosses (HP at/below pushes_at threshold)
        let entries: Vec<_> = self
            .data
            .entries
            .iter()
            .filter(|e| e.percent() > 0.0 && !e.is_pushed())
            .take(MAX_SUPPORTED_BOSSES)
            .cloned()
            .collect();

        // Nothing to render if no living bosses
        if entries.is_empty() {
            if self.config.dynamic_background {
                self.frame.begin_frame_with_content_height(0.0);
            } else {
                self.frame.begin_frame();
            }
            self.frame.end_frame();
            return;
        }

        // Calculate compression factor based on entries
        let compression = self.compression_factor(&entries);

        // Pre-compute content height, then begin frame with content-aware background
        let content_height = self.compute_content_height(&entries, compression);
        if self.config.dynamic_background {
            self.frame.begin_frame_with_content_height(content_height);
        } else {
            self.frame.begin_frame();
        }

        // Clamp font_scale to sensible range
        let font_scale = self.config.font_scale.clamp(0.3, 2.0);

        // Apply compression to entry-specific dimensions
        let padding = self.frame.scaled(BASE_PADDING);
        let bar_height = self.frame.scaled(BASE_BAR_HEIGHT) * compression;
        let entry_spacing = self.frame.scaled(BASE_ENTRY_SPACING) * compression;
        let label_bar_gap = self.frame.scaled(BASE_LABEL_BAR_GAP) * compression;
        let font_size = self.frame.scaled(BASE_FONT_SIZE) * compression * font_scale;
        let shield_bar_height = self.frame.scaled(BASE_SHIELD_BAR_HEIGHT) * compression;

        let bar_color = color_from_rgba(self.config.bar_color);
        let font_color = color_from_rgba(self.config.font_color);

        let content_width = width - padding * 2.0;
        let bar_radius = 4.0 * self.frame.scale_factor() * compression;
        let icon_size = bar_height * 0.6;
        let icon_spacing = 2.0;
        let time_font_size = icon_size * 0.38;

        let mut y = padding;

        for entry in &entries {
            let progress = entry.percent() / 100.0;

            // Find the next relevant HP marker (used for line + label below bar)
            let marker = Self::next_marker(entry);

            // Target name (rendered below the HP bar, if present and enabled)
            let target_info = if self.config.show_target {
                entry.target_name.as_ref().map(|t| format!("⌖ {}", t))
            } else {
                None
            };

            // ── Shield Bar (above HP bar, only when shields active) ─────
            if !entry.active_shields.is_empty() {
                // Use the first active shield for display (most common: single shield)
                let shield = &entry.active_shields[0];
                let shield_progress = if shield.total > 0 {
                    (shield.remaining as f32 / shield.total as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let shield_label = format!(
                    "{}: {}",
                    shield.label,
                    formatting::format_compact(shield.remaining, self.european_number_format)
                );
                let shield_font_size = font_size * 0.55;
                let shield_radius = bar_radius * 0.6;

                ProgressBar::new(&shield_label, shield_progress)
                    .with_fill_color(shield_bar_color())
                    .with_bg_color(colors::dps_bar_bg())
                    .with_text_color(font_color)
                    .with_text_glow()
                    .render(
                        &mut self.frame,
                        padding,
                        y,
                        content_width,
                        shield_bar_height,
                        shield_font_size,
                        shield_radius,
                    );

                y += shield_bar_height + label_bar_gap;
            }

            // ── HP Bar (inline: Name | HP | Percent) ────────────────────
            let health_text =
                formatting::format_compact(entry.current as i64, self.european_number_format);
            let percent_text = if self.config.show_percent {
                formatting::format_pct(entry.percent() as f64, self.european_number_format)
            } else {
                String::new()
            };

            let bar_font_size = font_size * 0.70;
            let bar_y = y;

            // Render bar background + fill (no inline text — we'll layer the
            // three columns ourselves so the name can scale down independently).
            ProgressBar::new("", progress)
                .with_fill_color(bar_color)
                .with_bg_color(colors::dps_bar_bg())
                .render(
                    &mut self.frame,
                    padding,
                    bar_y,
                    content_width,
                    bar_height,
                    bar_font_size,
                    bar_radius,
                );

            // Per-bar border outline (user-configurable colour, toggleable).
            if self.config.show_border {
                self.frame.stroke_rounded_rect(
                    padding,
                    bar_y,
                    content_width,
                    bar_height,
                    bar_radius,
                    0.8 * self.frame.scale_factor(),
                    color_from_rgba(self.config.border_color),
                );
            }

            // ── HP Marker Line (vertical line through the bar) ──────────
            if let Some((hp_pct, _)) = marker {
                let marker_x = padding + (hp_pct / 100.0) * content_width;
                let line_width = 2.0_f32;
                self.frame.fill_rect(
                    marker_x - line_width / 2.0,
                    bar_y,
                    line_width,
                    bar_height,
                    marker_line_color(),
                );
            }

            // Three-column inline text on top of the bar.
            // HP column is wide enough for "123.45M" without scaling; percent
            // column wide enough for "12.3%". Name takes the remainder and
            // scales down to fit if needed. When a target badge is present
            // we shrink the inline content width so the badge's bar-overlap
            // never collides with the right-aligned HP/percent text.
            let inline_text_pad = 6.0 * self.frame.scale_factor() * compression;
            let inline_col_gap = 8.0 * self.frame.scale_factor() * compression;

            // Pick a unified font size for Name + HP + Percent that respects
            // the configured `font_scale` (via `bar_font_size`) and only
            // shrinks when the three would otherwise overflow.
            //
            // Reservations are digit-normalized versions of the actual text
            // (each digit replaced with the widest glyph "8"), so the slot
            // is just-wide-enough for the current value's character count
            // rather than the absolute maximum. This avoids wasting space
            // when the boss's HP is e.g. "1.85M" — the reservation collapses
            // from "888.88M" down to "8.88M", giving the row more headroom
            // to grow with `font_scale`.
            let gap_count =
                self.config.show_hp_value as i32 + self.config.show_percent as i32;
            let fixed_padding =
                inline_text_pad * 2.0 + inline_col_gap * gap_count as f32;

            let hp_reserve = digit_normalize(&health_text);
            let pct_reserve = digit_normalize(&percent_text);

            let (name_w_at_bar, _) = self.frame.measure_text(&entry.name, bar_font_size);
            let (hp_w_at_bar, _) = if self.config.show_hp_value {
                self.frame.measure_text(&hp_reserve, bar_font_size)
            } else {
                (0.0, 0.0)
            };
            let (pct_w_at_bar, _) = if self.config.show_percent {
                self.frame.measure_text(&pct_reserve, bar_font_size)
            } else {
                (0.0, 0.0)
            };
            let total_at_bar = name_w_at_bar + hp_w_at_bar + pct_w_at_bar;

            // Default: respect bar_font_size (so font_scale takes effect).
            // Only shrink if the three texts plus padding actually overflow.
            let mut row_font_size = bar_font_size;
            if total_at_bar + fixed_padding > content_width && total_at_bar > 0.0 {
                let ratio = ((content_width - fixed_padding).max(0.0)) / total_at_bar;
                row_font_size = (bar_font_size * ratio).max(bar_font_size * 0.30);
            }

            // Re-measure the three slot widths at the final row_font_size so
            // they hug the right edge cleanly with no leftover empty space.
            let (hp_col_w, _) = if self.config.show_hp_value {
                self.frame.measure_text(&hp_reserve, row_font_size)
            } else {
                (0.0, 0.0)
            };
            let hp_section = if self.config.show_hp_value {
                inline_col_gap + hp_col_w
            } else {
                0.0
            };
            let (pct_col_w, _) = if self.config.show_percent {
                self.frame.measure_text(&pct_reserve, row_font_size)
            } else {
                (0.0, 0.0)
            };
            let pct_section = if self.config.show_percent {
                inline_col_gap + pct_col_w
            } else {
                0.0
            };
            let name_col_w = (content_width
                - inline_text_pad * 2.0
                - hp_section
                - pct_section)
                .max(0.0);

            // ── Target badge geometry (floating, rendered after the bar) ─
            // Badge font scales with the actual HP text (row_font_size) so
            // it stays proportional when the row shrank for a long name.
            let badge_info: Option<(String, f32, f32, f32, f32)> = target_info
                .as_ref()
                .map(|t| {
                    let badge_pad_x = 4.0 * self.frame.scale_factor() * compression;
                    let badge_pad_y = 1.5 * self.frame.scale_factor() * compression;
                    let base_badge_font = row_font_size * 0.50;
                    let max_text_w = (content_width * 0.45).max(40.0);
                    let badge_font = self.scaled_font_for_text(t, max_text_w, base_badge_font);
                    let (text_w, _) = self.frame.measure_text(t, badge_font);
                    let badge_w = text_w + badge_pad_x * 2.0;
                    let badge_h = badge_font + badge_pad_y * 2.0;
                    (t.clone(), badge_w, badge_h, badge_font, badge_pad_x)
                });

            let text_y = bar_y + bar_height / 2.0 + row_font_size / 3.0;

            // Name (left-aligned within its column). All three texts share
            // row_font_size — the iterative fit above guarantees they all fit.
            self.frame.draw_text_glowed(
                &entry.name,
                padding + inline_text_pad,
                text_y,
                row_font_size,
                font_color,
            );

            // HP value (right-aligned within its column)
            let hp_col_x = padding + inline_text_pad + name_col_w + inline_col_gap;
            if self.config.show_hp_value {
                let (hp_w, _) = self.frame.measure_text(&health_text, row_font_size);
                self.frame.draw_text_glowed(
                    &health_text,
                    hp_col_x + hp_col_w - hp_w,
                    text_y,
                    row_font_size,
                    font_color,
                );
            }

            // Percent (right-aligned within its column)
            if self.config.show_percent {
                // When HP is hidden, percent column abuts the name column
                // directly (no extra gap from the empty HP slot).
                let pct_col_x = if self.config.show_hp_value {
                    hp_col_x + hp_col_w + inline_col_gap
                } else {
                    padding + inline_text_pad + name_col_w + inline_col_gap
                };
                let (pct_w, _) = self.frame.measure_text(&percent_text, row_font_size);
                self.frame.draw_text_glowed(
                    &percent_text,
                    pct_col_x + pct_col_w - pct_w,
                    text_y,
                    row_font_size,
                    font_color,
                );
            }

            // ── Target Badge (half on / half off the bar's bottom-right) ─
            if let Some((badge_text, badge_w, badge_h, badge_font, badge_pad_x)) = &badge_info {
                let bw = *badge_w;
                let bh = *badge_h;
                let bfont = *badge_font;
                let bpad = *badge_pad_x;
                let badge_x = padding + content_width - bw;
                let badge_y = bar_y + bar_height - bh / 2.0;
                let badge_radius = bh / 2.0;
                let badge_bg =
                    tiny_skia::Color::from_rgba(0.10, 0.10, 0.10, 0.88).unwrap_or(Color::BLACK);
                self.frame.fill_rounded_rect(badge_x, badge_y, bw, bh, badge_radius, badge_bg);
                if self.config.show_border {
                    self.frame.stroke_rounded_rect(
                        badge_x,
                        badge_y,
                        bw,
                        bh,
                        badge_radius,
                        0.4 * self.frame.scale_factor(),
                        color_from_rgba(self.config.border_color),
                    );
                }
                let badge_text_y = badge_y + bh / 2.0 + bfont / 3.0;
                self.frame.draw_text_glowed(
                    badge_text,
                    badge_x + bpad,
                    badge_text_y,
                    bfont,
                    font_color,
                );
            }

            y += bar_height;
            // Badge floats — it overlaps subsequent rows visually but does
            // not push them down. No vertical reserve.

            // ── Icon + Marker Row (below bar) ──────────────────────────
            let entry_icons = self.data.boss_icons.get(&entry.entity_id);
            if entry_icons.is_some_and(|v| !v.is_empty()) {
                let icons = entry_icons.unwrap();
                let icon_y = y + 3.0;
                let mut icon_x = padding;

                for icon_entry in icons {
                    let drawn = if icon_entry.show_icon {
                        if let Some(ref img) = icon_entry.icon {
                            let (iw, ih, ref rgba) = **img;
                            self.frame.draw_image(rgba, iw, ih, icon_x, icon_y, icon_size, icon_size);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !drawn {
                        self.frame.fill_rounded_rect(
                            icon_x, icon_y, icon_size, icon_size, 2.0,
                            color_from_rgba(icon_entry.color),
                        );
                    }

                    // Clock wipe — dark overlay from top, shrinks as time remains
                    let overlay_h = icon_size * (1.0 - icon_entry.progress());
                    if overlay_h > 1.0 {
                        self.frame.fill_rect(
                            icon_x, icon_y, icon_size, overlay_h,
                            color_from_rgba([0, 0, 0, 140]),
                        );
                    }

                    self.frame.stroke_rounded_rect(
                        icon_x, icon_y, icon_size, icon_size, 2.0, 1.0, colors::white(),
                    );

                    let time_text = icon_entry.format_time(self.european_number_format);
                    let (tw, _) = self.frame.measure_text(&time_text, time_font_size);
                    let time_color = if icon_entry.remaining_secs <= 3.0 {
                        colors::effect_debuff()
                    } else {
                        colors::white()
                    };
                    self.frame.draw_text_glowed(
                        &time_text,
                        icon_x + (icon_size - tw) / 2.0,
                        icon_y + icon_size / 2.0 + time_font_size * 0.4,
                        time_font_size,
                        time_color,
                    );

                    icon_x += icon_size + icon_spacing;
                }

                // Marker text to the right of icons (same row, vertically centered)
                if let Some((hp_pct, label)) = marker {
                    let marker_font_size = row_font_size * 0.55;
                    let marker_label = format!("{}% {}", hp_pct as u32, label);
                    self.frame.draw_text_glowed(
                        &marker_label,
                        icon_x + icon_spacing,
                        icon_y + icon_size / 2.0 + marker_font_size * 0.4,
                        marker_font_size,
                        marker_line_color(),
                    );
                }

                y += icon_size + 6.0;
            } else if let Some((hp_pct, label)) = marker {
                // Left-aligned at the bar's content padding (no icons row to
                // anchor against).
                let marker_font_size = row_font_size * 0.55;
                let marker_label = format!("{}% {}", hp_pct as u32, label);
                self.frame.draw_text_glowed(
                    &marker_label,
                    padding,
                    y + marker_font_size + 1.0,
                    marker_font_size,
                    marker_line_color(),
                );
                y += marker_font_size + 2.0;
            }

            y += entry_spacing;
        }

        // End frame (resize indicator, commit)
        self.frame.end_frame();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Overlay Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Overlay for BossHealthOverlay {
    fn update_data(&mut self, data: OverlayData) -> bool {
        if let OverlayData::BossHealth(boss_data) = data {
            // When clear_after_combat is disabled, ignore empty clears
            // so the last boss health remains visible
            if boss_data.entries.is_empty() && !self.config.clear_after_combat {
                return false;
            }

            // Active effect icons tick every frame — render every frame while present.
            // Track total count so the trailing edge (last icon expires) forces one
            // final render to erase the stale "0.0" countdown text.
            let new_icon_count: usize =
                boss_data.boss_icons.values().map(|v| v.len()).sum();
            let icons_changed = new_icon_count != self.last_icon_count;
            let has_active_effects = new_icon_count > 0;
            self.last_icon_count = new_icon_count;

            // Re-render when HP or shield state changed. Per-shield `remaining` is
            // included so absorbing damage smoothly redraws the shield bar without
            // requiring a count change.
            let new_sig: Vec<(i32, i32, Vec<i64>)> = boss_data
                .entries
                .iter()
                .map(|e| {
                    (
                        e.current,
                        e.max,
                        e.active_shields.iter().map(|s| s.remaining).collect(),
                    )
                })
                .collect();
            let hp_changed = new_sig != self.last_hp_sig;
            self.last_hp_sig = new_sig;

            self.set_data(boss_data);
            has_active_effects || icons_changed || hp_changed
        } else {
            false
        }
    }

    fn update_config(&mut self, config: OverlayConfigUpdate) {
        if let OverlayConfigUpdate::BossHealth(boss_config, alpha, european) = config {
            self.set_config(boss_config);
            self.set_background_alpha(alpha);
            self.european_number_format = european;
        }
    }

    fn render(&mut self) {
        BossHealthOverlay::render(self);
    }

    fn poll_events(&mut self) -> bool {
        self.frame.poll_events()
    }

    fn frame(&self) -> &OverlayFrame {
        &self.frame
    }

    fn frame_mut(&mut self) -> &mut OverlayFrame {
        &mut self.frame
    }
}
