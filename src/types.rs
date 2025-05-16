use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Tab {
    #[default]
    LogWeight,
    LogExercise,
    WeightProgress,
    ExerciseProgress,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum DietPhase {
    #[default]
    Bulk,
    Cut,
    Maintain,
}

impl Display for DietPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum ExerciseMetric {
    #[default]
    Weight,
}

impl Display for ExerciseMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
} 