use egui::Color32;
use crate::data::CameraPose;

#[derive(Clone, Debug, PartialEq)]
pub enum PlotField {
    PositionX,
    PositionY,
    PositionZ,
    Speed,
}

impl PlotField {
    pub fn label(&self) -> &str {
        match self {
            PlotField::PositionX => "X",
            PlotField::PositionY => "Y",
            PlotField::PositionZ => "Z",
            PlotField::Speed     => "Speed",
        }
    }

    pub fn value_at(&self, pose: &CameraPose, prev: Option<&CameraPose>) -> f64 {
        match self {
            PlotField::PositionX => pose.position[0] as f64,
            PlotField::PositionY => pose.position[1] as f64,
            PlotField::PositionZ => pose.position[2] as f64,
            PlotField::Speed => {
                if let Some(p) = prev {
                    let dt = pose.timestamp - p.timestamp;
                    if dt > 0.0 {
                        let dx = (pose.position[0] - p.position[0]) as f64;
                        let dy = (pose.position[1] - p.position[1]) as f64;
                        let dz = (pose.position[2] - p.position[2]) as f64;
                        (dx * dx + dy * dy + dz * dz).sqrt() / dt
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
        }
    }

    pub fn default_color(&self) -> Color32 {
        match self {
            PlotField::PositionX => Color32::from_rgb(255, 80,  80),
            PlotField::PositionY => Color32::from_rgb(80,  200, 80),
            PlotField::PositionZ => Color32::from_rgb(80,  140, 255),
            PlotField::Speed     => Color32::from_rgb(255, 180, 50),
        }
    }

    pub fn all() -> &'static [PlotField] {
        &[
            PlotField::PositionX,
            PlotField::PositionY,
            PlotField::PositionZ,
            PlotField::Speed,
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimePlotConfig {
    pub field: PlotField,
    pub color: Color32,
}

impl TimePlotConfig {
    pub fn new(field: PlotField) -> Self {
        let color = field.default_color();
        Self { field, color }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaneKind {
    View3D,
    InfoPanel,
    MiniMap,
    PositionPlot,
    SpeedPlot,
    TimePlot(TimePlotConfig),
    PlotPicker,
}

impl PaneKind {
    pub fn title(&self) -> &str {
        match self {
            PaneKind::View3D        => "3D View",
            PaneKind::InfoPanel     => "Info",
            PaneKind::MiniMap       => "Map",
            PaneKind::PositionPlot  => "Position",
            PaneKind::SpeedPlot     => "Speed",
            PaneKind::TimePlot(cfg) => cfg.field.label(),
            PaneKind::PlotPicker    => "Plots",
        }
    }
}
