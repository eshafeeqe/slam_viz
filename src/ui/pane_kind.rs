#[derive(Clone, Debug, PartialEq)]
pub enum PaneKind {
    View3D,
    InfoPanel,
    MiniMap,
    PositionPlot,
    SpeedPlot,
}

impl PaneKind {
    pub fn title(&self) -> &str {
        match self {
            PaneKind::View3D       => "3D View",
            PaneKind::InfoPanel    => "Info",
            PaneKind::MiniMap      => "Map",
            PaneKind::PositionPlot => "Position",
            PaneKind::SpeedPlot    => "Speed",
        }
    }
}
