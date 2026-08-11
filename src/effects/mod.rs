pub mod beams;
pub mod binarypath;
pub mod blackhole;
pub mod bouncyballs;
pub mod bubbles;
pub mod burn;
pub mod colorshift;
pub mod crumble;
pub mod decrypt;
pub mod errorcorrect;
pub mod expand;
pub mod fireworks;
pub mod highlight;
pub mod laseretch;
pub mod matrix;
pub mod middleout;
pub mod orbittingvolley;
pub mod overflow;
pub mod pour;
pub mod print;
pub mod rain;
pub mod randomsequence;
pub mod rings;
pub mod scattered;
pub mod slice;
pub mod slide;
pub mod smoke;
pub mod spotlights;
pub mod spray;
pub mod swarm;
pub mod sweep;
pub mod synthgrid;
pub mod thunderstorm;
pub mod unstable;
pub mod vhstape;
pub mod waves;
pub mod wipe;
pub mod wormhole;

// Re-exports for test convenience (use crate::effects::*)
#[cfg(test)]
pub use {
    beams::BeamsEffect, binarypath::BinaryPathEffect, blackhole::BlackholeEffect,
    bouncyballs::BouncyBallsEffect, bubbles::BubblesEffect, burn::BurnEffect,
    colorshift::ColorShiftEffect, crumble::CrumbleEffect, decrypt::DecryptEffect,
    errorcorrect::ErrorCorrectEffect, expand::ExpandEffect, fireworks::FireworksEffect,
    highlight::HighlightEffect, laseretch::LaserEtchEffect, matrix::MatrixEffect,
    middleout::MiddleOutEffect, orbittingvolley::OrbittingVolleyEffect, overflow::OverflowEffect,
    pour::PourEffect, print::PrintEffect, rain::RainEffect, randomsequence::RandomSequenceEffect,
    rings::RingsEffect, scattered::ScatteredEffect, slice::SliceEffect, slide::SlideEffect,
    smoke::SmokeEffect, spotlights::SpotlightsEffect, spray::SprayEffect, swarm::SwarmEffect,
    sweep::SweepEffect, synthgrid::SynthGridEffect, thunderstorm::ThunderstormEffect,
    unstable::UnstableEffect, vhstape::VHSTapeEffect, waves::WavesEffect, wipe::WipeEffect,
    wormhole::WormholeEffect,
};

use crate::engine::Grid;

/// Effects that are RTTE originals, not present in TTE.
pub const EXCLUSIVE_NAMES: &[&str] = &["wormhole"];

/// Common trait for all effects.
pub trait Effect {
    fn tick(&mut self, grid: &mut Grid) -> bool;
}

/// Effect registry entry: metadata + constructor.
pub struct EffectInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub extra_effect: bool,
    pub create: fn(&Grid) -> Box<dyn Effect>,
}

// Implement Effect for every effect type, delegating to their inherent tick().
macro_rules! impl_effect {
    ($($ty:path),* $(,)?) => {
        $(impl Effect for $ty {
            fn tick(&mut self, grid: &mut Grid) -> bool {
                self.tick(grid)
            }
        })*
    };
}

impl_effect!(
    beams::BeamsEffect,
    binarypath::BinaryPathEffect,
    blackhole::BlackholeEffect,
    bouncyballs::BouncyBallsEffect,
    bubbles::BubblesEffect,
    burn::BurnEffect,
    colorshift::ColorShiftEffect,
    crumble::CrumbleEffect,
    decrypt::DecryptEffect,
    errorcorrect::ErrorCorrectEffect,
    expand::ExpandEffect,
    fireworks::FireworksEffect,
    highlight::HighlightEffect,
    laseretch::LaserEtchEffect,
    matrix::MatrixEffect,
    middleout::MiddleOutEffect,
    orbittingvolley::OrbittingVolleyEffect,
    overflow::OverflowEffect,
    pour::PourEffect,
    print::PrintEffect,
    rain::RainEffect,
    randomsequence::RandomSequenceEffect,
    rings::RingsEffect,
    scattered::ScatteredEffect,
    slice::SliceEffect,
    slide::SlideEffect,
    smoke::SmokeEffect,
    spotlights::SpotlightsEffect,
    spray::SprayEffect,
    swarm::SwarmEffect,
    sweep::SweepEffect,
    synthgrid::SynthGridEffect,
    thunderstorm::ThunderstormEffect,
    unstable::UnstableEffect,
    vhstape::VHSTapeEffect,
    waves::WavesEffect,
    wipe::WipeEffect,
    wormhole::WormholeEffect,
);

/// Registry of all effects — metadata + constructor, sourced from each module.
/// Each module must define NAME, DESCRIPTION, and EXCLUSIVE constants.
macro_rules! register_effects {
    ($($mod:ident :: $ty:ident),* $(,)?) => {
        pub const ALL_EFFECTS: &[EffectInfo] = &[
            $(EffectInfo {
                name: $mod::NAME,
                description: $mod::DESCRIPTION,
                extra_effect: $mod::EXTRA_EFFECT,
                create: |grid| Box::new($mod::$ty::new(grid)),
            }),*
        ];
    };
}

register_effects!(
    beams::BeamsEffect,
    binarypath::BinaryPathEffect,
    blackhole::BlackholeEffect,
    bouncyballs::BouncyBallsEffect,
    bubbles::BubblesEffect,
    burn::BurnEffect,
    colorshift::ColorShiftEffect,
    crumble::CrumbleEffect,
    decrypt::DecryptEffect,
    errorcorrect::ErrorCorrectEffect,
    expand::ExpandEffect,
    fireworks::FireworksEffect,
    highlight::HighlightEffect,
    laseretch::LaserEtchEffect,
    matrix::MatrixEffect,
    middleout::MiddleOutEffect,
    orbittingvolley::OrbittingVolleyEffect,
    overflow::OverflowEffect,
    pour::PourEffect,
    print::PrintEffect,
    rain::RainEffect,
    randomsequence::RandomSequenceEffect,
    rings::RingsEffect,
    scattered::ScatteredEffect,
    slice::SliceEffect,
    slide::SlideEffect,
    smoke::SmokeEffect,
    spotlights::SpotlightsEffect,
    spray::SprayEffect,
    swarm::SwarmEffect,
    sweep::SweepEffect,
    synthgrid::SynthGridEffect,
    thunderstorm::ThunderstormEffect,
    unstable::UnstableEffect,
    vhstape::VHSTapeEffect,
    waves::WavesEffect,
    wipe::WipeEffect,
    wormhole::WormholeEffect,
);
