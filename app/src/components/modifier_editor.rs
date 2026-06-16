//! Modifier editor component for the effect editor
//!
//! Provides inline editing of `EffectModifier` entries on an `EffectDefinition`.
//! Each modifier has a trigger type, duration adjustment, and optional constraints.

use dioxus::prelude::*;

use super::encounter_editor::triggers::{
    AbilitySelectorEditor, EffectSelectorEditor,
};
use crate::types::{
    AbilitySelector, ChargeDirection, EffectModifier, EffectSelector, MitigationType, Trigger,
};

// ─────────────────────────────────────────────────────────────────────────────
// Modifier Trigger Type (UI discriminant)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Default)]
enum ModifierTriggerType {
    #[default]
    AbilityCast,
    DamageTaken,
    DamageDealt,
    HealingTaken,
    EffectApplied,
    EffectRemoved,
    ChargesChanged,
    SelfChargesChanged,
}

impl ModifierTriggerType {
    fn label(self) -> &'static str {
        match self {
            Self::AbilityCast => "Ability Cast",
            Self::DamageTaken => "Damage Taken",
            Self::DamageDealt => "Damage Dealt",
            Self::HealingTaken => "Healing Taken",
            Self::EffectApplied => "Effect Applied",
            Self::EffectRemoved => "Effect Removed",
            Self::ChargesChanged => "Charges Changed",
            Self::SelfChargesChanged => "Self Charges Changed",
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::AbilityCast,
            Self::DamageTaken,
            Self::DamageDealt,
            Self::HealingTaken,
            Self::EffectApplied,
            Self::EffectRemoved,
            Self::ChargesChanged,
            Self::SelfChargesChanged,
        ]
    }

    fn from_trigger(trigger: &Trigger) -> Self {
        match trigger {
            Trigger::AbilityCast { .. } => Self::AbilityCast,
            Trigger::DamageTaken { .. } => Self::DamageTaken,
            Trigger::DamageDealt { .. } => Self::DamageDealt,
            Trigger::HealingTaken { .. } => Self::HealingTaken,
            Trigger::EffectApplied { .. } => Self::EffectApplied,
            Trigger::EffectRemoved { .. } => Self::EffectRemoved,
            Trigger::ChargesChanged { .. } => Self::ChargesChanged,
            Trigger::SelfChargesChanged { .. } => Self::SelfChargesChanged,
            _ => Self::AbilityCast,
        }
    }

    fn default_trigger(self) -> Trigger {
        match self {
            Self::AbilityCast => Trigger::AbilityCast {
                abilities: vec![],
                source: Default::default(),
                target: Default::default(),
            },
            Self::DamageTaken => Trigger::DamageTaken {
                abilities: vec![],
                source: Default::default(),
                target: Default::default(),
                mitigation: vec![],
            },
            Self::DamageDealt => Trigger::DamageDealt {
                abilities: vec![],
                source: Default::default(),
                target: Default::default(),
                mitigation: vec![],
            },
            Self::HealingTaken => Trigger::HealingTaken {
                abilities: vec![],
                source: Default::default(),
                target: Default::default(),
            },
            Self::EffectApplied => Trigger::EffectApplied {
                effects: vec![],
                source: Default::default(),
                target: Default::default(),
            },
            Self::EffectRemoved => Trigger::EffectRemoved {
                effects: vec![],
                source: Default::default(),
                target: Default::default(),
            },
            Self::ChargesChanged => Trigger::ChargesChanged {
                effects: vec![],
                direction: None,
            },
            Self::SelfChargesChanged => Trigger::SelfChargesChanged { direction: None },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Modifier List Editor
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ModifierListEditorProps {
    pub modifiers: Vec<EffectModifier>,
    pub on_change: EventHandler<Vec<EffectModifier>>,
}

#[component]
pub fn ModifierListEditor(props: ModifierListEditorProps) -> Element {
    let modifiers = props.modifiers.clone();

    rsx! {
        div { class: "form-card",
            div { class: "form-card-header",
                i { class: "fa-solid fa-sliders" }
                span { "Modifiers" }
                button {
                    class: "btn-icon-sm",
                    title: "Add modifier",
                    style: "margin-left: auto;",
                    onclick: {
                        let modifiers = modifiers.clone();
                        let on_change = props.on_change.clone();
                        move |_| {
                            let mut mods = modifiers.clone();
                            mods.push(EffectModifier {
                                trigger: Trigger::DamageTaken {
                                    abilities: vec![],
                                    source: Default::default(),
                                    target: Default::default(),
                                    mitigation: vec![],
                                },
                                adjust_duration_secs: -1.0,
                                requires_crit: false,
                                refill_duration: false,
                                icd_secs: None,
                                max_duration_secs: None,
                            });
                            on_change.call(mods);
                        }
                    },
                    i { class: "fa-solid fa-plus" }
                }
            }
            div { class: "form-card-content",
                if modifiers.is_empty() {
                    div { class: "text-muted text-sm", style: "padding: 4px 0;",
                        "No modifiers configured. Add one to reactively adjust this effect's duration or charges when events occur."
                    }
                }
                for (idx, modifier) in modifiers.iter().enumerate() {
                    {
                        let on_change = props.on_change.clone();
                        let all_modifiers = modifiers.clone();
                        rsx! {
                            SingleModifierEditor {
                                key: "{idx}",
                                modifier: modifier.clone(),
                                index: idx,
                                on_update: {
                                    let all_modifiers = all_modifiers.clone();
                                    let on_change = on_change.clone();
                                    move |updated: EffectModifier| {
                                        let mut mods = all_modifiers.clone();
                                        mods[idx] = updated;
                                        on_change.call(mods);
                                    }
                                },
                                on_remove: {
                                    let all_modifiers = all_modifiers.clone();
                                    let on_change = on_change.clone();
                                    move |_| {
                                        let mut mods = all_modifiers.clone();
                                        mods.remove(idx);
                                        on_change.call(mods);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Single Modifier Editor
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct SingleModifierEditorProps {
    modifier: EffectModifier,
    index: usize,
    on_update: EventHandler<EffectModifier>,
    on_remove: EventHandler<()>,
}

#[component]
fn SingleModifierEditor(props: SingleModifierEditorProps) -> Element {
    let trigger_type = ModifierTriggerType::from_trigger(&props.modifier.trigger);
    let modifier = props.modifier.clone();

    rsx! {
        div {
            class: "modifier-entry",
            style: "border: 1px solid var(--border-color, #333); border-radius: 4px; padding: 8px; margin-bottom: 6px;",

            // Header row: trigger type + remove button
            div { class: "flex items-center gap-sm", style: "margin-bottom: 6px;",
                select {
                    class: "select-inline",
                    style: "flex: 1;",
                    value: "{trigger_type.label()}",
                    onchange: {
                        let modifier = modifier.clone();
                        let on_update = props.on_update.clone();
                        move |e: Event<FormData>| {
                            let new_type = match e.value().as_str() {
                                "Ability Cast" => ModifierTriggerType::AbilityCast,
                                "Damage Taken" => ModifierTriggerType::DamageTaken,
                                "Damage Dealt" => ModifierTriggerType::DamageDealt,
                                "Healing Taken" => ModifierTriggerType::HealingTaken,
                                "Effect Applied" => ModifierTriggerType::EffectApplied,
                                "Effect Removed" => ModifierTriggerType::EffectRemoved,
                                "Charges Changed" => ModifierTriggerType::ChargesChanged,
                                "Self Charges Changed" => ModifierTriggerType::SelfChargesChanged,
                                _ => return,
                            };
                            let mut m = modifier.clone();
                            m.trigger = new_type.default_trigger();
                            on_update.call(m);
                        }
                    },
                    for tt in ModifierTriggerType::all() {
                        option { value: "{tt.label()}", selected: *tt == trigger_type, "{tt.label()}" }
                    }
                }
                button {
                    class: "btn-icon-sm btn-danger",
                    title: "Remove modifier",
                    onclick: {
                        let on_remove = props.on_remove.clone();
                        move |_| on_remove.call(())
                    },
                    i { class: "fa-solid fa-trash" }
                }
            }

            // Trigger-specific fields
            {render_trigger_fields(&modifier, &props.on_update)}

            // Common modifier fields
            div { class: "form-row-hz",
                label { "Duration Adjust (s)" }
                input {
                    r#type: "number",
                    class: "input-number",
                    step: "0.1",
                    value: "{modifier.adjust_duration_secs}",
                    onchange: {
                        let modifier = modifier.clone();
                        let on_update = props.on_update.clone();
                        move |e: Event<FormData>| {
                            let mut m = modifier.clone();
                            m.adjust_duration_secs = e.value().parse().unwrap_or(0.0);
                            on_update.call(m);
                        }
                    }
                }
            }

            // Requires Crit (only for DamageTaken/HealingTaken)
            if matches!(trigger_type, ModifierTriggerType::DamageTaken | ModifierTriggerType::DamageDealt | ModifierTriggerType::HealingTaken) {
                div { class: "form-row-hz",
                    label { "Requires Critical Hit" }
                    input {
                        r#type: "checkbox",
                        checked: modifier.requires_crit,
                        onchange: {
                            let modifier = modifier.clone();
                            let on_update = props.on_update.clone();
                            move |e: Event<FormData>| {
                                let mut m = modifier.clone();
                                m.requires_crit = e.checked();
                                on_update.call(m);
                            }
                        }
                    }
                }
            }

            // Refill Duration
            div { class: "form-row-hz",
                label {
                    "Refill Duration"
                    span { class: "help-icon", title: "Reset remaining time to the effect's base duration on each proc instead of adjusting by a fixed delta", "?" }
                }
                input {
                    r#type: "checkbox",
                    checked: modifier.refill_duration,
                    onchange: {
                        let modifier = modifier.clone();
                        let on_update = props.on_update.clone();
                        move |e: Event<FormData>| {
                            let mut m = modifier.clone();
                            m.refill_duration = e.checked();
                            on_update.call(m);
                        }
                    }
                }
            }

            // ICD
            div { class: "form-row-hz",
                label {
                    "ICD (s)"
                    span { class: "help-icon", title: "Internal cooldown — minimum seconds between activations", "?" }
                }
                input {
                    r#type: "number",
                    class: "input-number",
                    step: "0.1",
                    min: "0",
                    placeholder: "None",
                    value: "{modifier.icd_secs.map(|v| v.to_string()).unwrap_or_default()}",
                    onchange: {
                        let modifier = modifier.clone();
                        let on_update = props.on_update.clone();
                        move |e: Event<FormData>| {
                            let mut m = modifier.clone();
                            m.icd_secs = e.value().parse::<f32>().ok().filter(|v| *v > 0.0);
                            on_update.call(m);
                        }
                    }
                }
            }

            // Max Duration
            div { class: "form-row-hz",
                label { "Max Duration (s)" }
                input {
                    r#type: "number",
                    class: "input-number",
                    step: "0.5",
                    min: "0",
                    placeholder: "None",
                    value: "{modifier.max_duration_secs.map(|v| v.to_string()).unwrap_or_default()}",
                    onchange: {
                        let modifier = modifier.clone();
                        let on_update = props.on_update.clone();
                        move |e: Event<FormData>| {
                            let mut m = modifier.clone();
                            m.max_duration_secs = e.value().parse::<f32>().ok().filter(|v| *v > 0.0);
                            on_update.call(m);
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trigger-specific field rendering
// ─────────────────────────────────────────────────────────────────────────────

fn render_trigger_fields(modifier: &EffectModifier, on_update: &EventHandler<EffectModifier>) -> Element {
    match &modifier.trigger {
        Trigger::AbilityCast { abilities, .. } => {
            let abilities = abilities.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                AbilitySelectorEditor {
                    label: "Abilities",
                    selectors: abilities,
                    on_change: move |new_abs: Vec<AbilitySelector>| {
                        let mut m = modifier.clone();
                        m.trigger = Trigger::AbilityCast {
                            abilities: new_abs,
                            source: Default::default(),
                            target: Default::default(),
                        };
                        on_update.call(m);
                    }
                }
            }
        }
        Trigger::DamageTaken { abilities, mitigation, .. } => {
            let abilities = abilities.clone();
            let mitigation = mitigation.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                AbilitySelectorEditor {
                    label: "Abilities",
                    selectors: abilities.clone(),
                    on_change: {
                        let modifier = modifier.clone();
                        let on_update = on_update.clone();
                        let mitigation = mitigation.clone();
                        move |new_abs: Vec<AbilitySelector>| {
                            let mut m = modifier.clone();
                            m.trigger = Trigger::DamageTaken {
                                abilities: new_abs,
                                source: Default::default(),
                                target: Default::default(),
                                mitigation: mitigation.clone(),
                            };
                            on_update.call(m);
                        }
                    }
                }
                div { class: "form-row-hz",
                    label { "Mitigation Filter" }
                    div { class: "flex flex-wrap gap-xs",
                        for mit_type in MitigationType::ALL {
                            label { class: "flex items-center gap-xs text-sm",
                                input {
                                    r#type: "checkbox",
                                    checked: mitigation.contains(mit_type),
                                    onchange: {
                                        let modifier = modifier.clone();
                                        let on_update = on_update.clone();
                                        let mit = *mit_type;
                                        let abilities = abilities.clone();
                                        let mitigation = mitigation.clone();
                                        move |e: Event<FormData>| {
                                            let mut mits = mitigation.clone();
                                            if e.checked() {
                                                if !mits.contains(&mit) { mits.push(mit); }
                                            } else {
                                                mits.retain(|m| *m != mit);
                                            }
                                            let mut m = modifier.clone();
                                            m.trigger = Trigger::DamageTaken {
                                                abilities: abilities.clone(),
                                                source: Default::default(),
                                                target: Default::default(),
                                                mitigation: mits,
                                            };
                                            on_update.call(m);
                                        }
                                    }
                                }
                                span { "{mit_type.display_name()}" }
                            }
                        }
                    }
                }
            }
        }
        Trigger::DamageDealt { abilities, mitigation, .. } => {
            let abilities = abilities.clone();
            let mitigation = mitigation.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                AbilitySelectorEditor {
                    label: "Abilities",
                    selectors: abilities.clone(),
                    on_change: {
                        let modifier = modifier.clone();
                        let on_update = on_update.clone();
                        let mitigation = mitigation.clone();
                        move |new_abs: Vec<AbilitySelector>| {
                            let mut m = modifier.clone();
                            m.trigger = Trigger::DamageDealt {
                                abilities: new_abs,
                                source: Default::default(),
                                target: Default::default(),
                                mitigation: mitigation.clone(),
                            };
                            on_update.call(m);
                        }
                    }
                }
                div { class: "form-row-hz",
                    label { "Mitigation Filter" }
                    div { class: "flex flex-wrap gap-xs",
                        for mit_type in MitigationType::ALL {
                            label { class: "flex items-center gap-xs text-sm",
                                input {
                                    r#type: "checkbox",
                                    checked: mitigation.contains(mit_type),
                                    onchange: {
                                        let modifier = modifier.clone();
                                        let on_update = on_update.clone();
                                        let mit = *mit_type;
                                        let abilities = abilities.clone();
                                        let mitigation = mitigation.clone();
                                        move |e: Event<FormData>| {
                                            let mut mits = mitigation.clone();
                                            if e.checked() {
                                                if !mits.contains(&mit) { mits.push(mit); }
                                            } else {
                                                mits.retain(|m| *m != mit);
                                            }
                                            let mut m = modifier.clone();
                                            m.trigger = Trigger::DamageDealt {
                                                abilities: abilities.clone(),
                                                source: Default::default(),
                                                target: Default::default(),
                                                mitigation: mits,
                                            };
                                            on_update.call(m);
                                        }
                                    }
                                }
                                span { "{mit_type.display_name()}" }
                            }
                        }
                    }
                }
            }
        }
        Trigger::HealingTaken { abilities, .. } => {
            let abilities = abilities.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                AbilitySelectorEditor {
                    label: "Abilities",
                    selectors: abilities,
                    on_change: move |new_abs: Vec<AbilitySelector>| {
                        let mut m = modifier.clone();
                        m.trigger = Trigger::HealingTaken {
                            abilities: new_abs,
                            source: Default::default(),
                            target: Default::default(),
                        };
                        on_update.call(m);
                    }
                }
            }
        }
        Trigger::EffectApplied { effects, .. } => {
            let effects = effects.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                EffectSelectorEditor {
                    label: "Effects",
                    selectors: effects,
                    on_change: move |new_eff: Vec<EffectSelector>| {
                        let mut m = modifier.clone();
                        m.trigger = Trigger::EffectApplied {
                            effects: new_eff,
                            source: Default::default(),
                            target: Default::default(),
                        };
                        on_update.call(m);
                    }
                }
            }
        }
        Trigger::EffectRemoved { effects, .. } => {
            let effects = effects.clone();
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                EffectSelectorEditor {
                    label: "Effects",
                    selectors: effects,
                    on_change: move |new_eff: Vec<EffectSelector>| {
                        let mut m = modifier.clone();
                        m.trigger = Trigger::EffectRemoved {
                            effects: new_eff,
                            source: Default::default(),
                            target: Default::default(),
                        };
                        on_update.call(m);
                    }
                }
            }
        }
        Trigger::ChargesChanged { effects, direction } => {
            let effects = effects.clone();
            let direction = *direction;
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                EffectSelectorEditor {
                    label: "Effects",
                    selectors: effects.clone(),
                    on_change: {
                        let modifier = modifier.clone();
                        let on_update = on_update.clone();
                        let direction = direction;
                        move |new_eff: Vec<EffectSelector>| {
                            let mut m = modifier.clone();
                            m.trigger = Trigger::ChargesChanged {
                                effects: new_eff,
                                direction,
                            };
                            on_update.call(m);
                        }
                    }
                }
                {render_direction_select(direction, &modifier, &on_update, effects)}
            }
        }
        Trigger::SelfChargesChanged { direction } => {
            let direction = *direction;
            let modifier = modifier.clone();
            let on_update = on_update.clone();
            rsx! {
                div { class: "form-row-hz",
                    label { "Direction" }
                    select {
                        class: "select-inline",
                        value: "{direction_label(direction)}",
                        onchange: move |e: Event<FormData>| {
                            let dir = match e.value().as_str() {
                                "Increased" => Some(ChargeDirection::Increased),
                                "Decreased" => Some(ChargeDirection::Decreased),
                                "Neutral" => Some(ChargeDirection::Neutral),
                                _ => None,
                            };
                            let mut m = modifier.clone();
                            m.trigger = Trigger::SelfChargesChanged { direction: dir };
                            on_update.call(m);
                        },
                        option { value: "Any", selected: direction.is_none(), "Any" }
                        option { value: "Increased", selected: direction == Some(ChargeDirection::Increased), "Increased" }
                        option { value: "Decreased", selected: direction == Some(ChargeDirection::Decreased), "Decreased" }
                        option { value: "Neutral", selected: direction == Some(ChargeDirection::Neutral), "Neutral" }
                    }
                }
            }
        }
        _ => rsx! {},
    }
}

fn render_direction_select(
    direction: Option<ChargeDirection>,
    modifier: &EffectModifier,
    on_update: &EventHandler<EffectModifier>,
    effects: Vec<EffectSelector>,
) -> Element {
    let modifier = modifier.clone();
    let on_update = on_update.clone();
    rsx! {
        div { class: "form-row-hz",
            label { "Direction" }
            select {
                class: "select-inline",
                value: "{direction_label(direction)}",
                onchange: move |e: Event<FormData>| {
                    let dir = match e.value().as_str() {
                        "Increased" => Some(ChargeDirection::Increased),
                        "Decreased" => Some(ChargeDirection::Decreased),
                        "Neutral" => Some(ChargeDirection::Neutral),
                        _ => None,
                    };
                    let mut m = modifier.clone();
                    m.trigger = Trigger::ChargesChanged {
                        effects: effects.clone(),
                        direction: dir,
                    };
                    on_update.call(m);
                },
                option { value: "Any", selected: direction.is_none(), "Any" }
                option { value: "Increased", selected: direction == Some(ChargeDirection::Increased), "Increased" }
                option { value: "Decreased", selected: direction == Some(ChargeDirection::Decreased), "Decreased" }
                option { value: "Neutral", selected: direction == Some(ChargeDirection::Neutral), "Neutral" }
            }
        }
    }
}

fn direction_label(direction: Option<ChargeDirection>) -> &'static str {
    match direction {
        Some(ChargeDirection::Increased) => "Increased",
        Some(ChargeDirection::Decreased) => "Decreased",
        Some(ChargeDirection::Neutral) => "Neutral",
        None => "Any",
    }
}
