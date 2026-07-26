//! View kinds catalog (P6 Stage C).

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    ArchitectureMap,
    ImpactCone,
    SlicePath,
    PackMap,
    HotspotHeat,
    LayeringViolations,
    AmbiguityHeat,
}

impl ViewKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArchitectureMap => "architecture_map",
            Self::ImpactCone => "impact_cone",
            Self::SlicePath => "slice_path",
            Self::PackMap => "pack_map",
            Self::HotspotHeat => "hotspot_heat",
            Self::LayeringViolations => "layering_violations",
            Self::AmbiguityHeat => "ambiguity_heat",
        }
    }

    pub fn default_layout(self) -> &'static str {
        match self {
            Self::ArchitectureMap | Self::LayeringViolations => "layered",
            Self::ImpactCone | Self::AmbiguityHeat | Self::HotspotHeat => "radial",
            Self::SlicePath | Self::PackMap => "path",
        }
    }
}

impl FromStr for ViewKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "architecture_map" | "architecture" => Self::ArchitectureMap,
            "impact_cone" | "impact" => Self::ImpactCone,
            "slice_path" | "slice" => Self::SlicePath,
            "pack_map" | "pack" => Self::PackMap,
            "hotspot_heat" | "hotspot" => Self::HotspotHeat,
            "layering_violations" | "layering" => Self::LayeringViolations,
            "ambiguity_heat" | "ambiguity" => Self::AmbiguityHeat,
            other => return Err(format!("unknown view_kind '{other}'")),
        })
    }
}
