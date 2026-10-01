use std::{cmp, f64::consts::PI};

use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{strain_decay, StrainSkill},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::{
        difficulty::{bpm_to_milliseconds, logistic, milliseconds_to_bpm},
        strains_vec::StrainsVec,
    },
};

use super::strain::OsuStrainSkill;

define_skill! {
    #[derive(Clone)]
    pub struct Stamina: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
    }
}

/// Values of [`Stamina`] as used by ppplus-csr bindings and tooling.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct StaminaSkillOutput {
    /// Star rating of the skill.
    pub stars: f64,
    /// The un-square-rooted difficulty value.
    pub difficulty_value: f64,
    /// Weighted amount of strains that are considered difficult.
    pub difficult_strain_count: f64,
    /// Sum of all accumulated object strains.
    pub strain_sum: f64,
}

impl Stamina {
    const SKILL_MULTIPLIER: f64 = 2600.0 * 0.3;
    const STRAIN_DECAY_BASE: f64 = 0.45;

    /// Collect the values of this skill that are relevant for ppplus-csr.
    ///
    /// This is a convenience for bindings and tooling; it does not influence
    /// the regular difficulty calculation.
    pub fn skill_output(&self) -> StaminaSkillOutput {
        let difficulty_value = self.cloned_difficulty_value();

        StaminaSkillOutput {
            stars: difficulty_value.sqrt() * super::super::DIFFICULTY_MULTIPLIER,
            difficulty_value,
            difficult_strain_count: self.count_top_weighted_strains(difficulty_value),
            strain_sum: self.strain_skill_object_strains.iter().copied().sum(),
        }
    }

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        _objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.strain_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += StaminaEvaluator::evaluate_diff_of(
            curr,
        ) * Self::SKILL_MULTIPLIER;

        self.current_strain
    }
}

impl OsuStrainSkill for Stamina {}

struct StaminaEvaluator;

impl StaminaEvaluator {

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
    ) -> f64 {
        let ms = curr.last_two_strain_time / 2.0;
        
        let tap_value = 2.0 / (ms - 20.0);
        let stream_value = 1.0 / (ms - 20.0);

        (1.0 - curr.flow) * tap_value + curr.flow * stream_value
    }
}
