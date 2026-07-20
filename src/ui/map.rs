use egui::{Context, Response, Ui};
use walkers::{
    HttpTiles, Map, MapMemory, Plugin, Position, Projector, TileId, Tiles, lat_lon,
    sources::{Attribution, TileSource},
};
use super::theme;

pub const STORAGE_KEY: &str = "map_view";

#[derive(Clone)]
pub struct VehicleMarker {
    pub position: Position,
    pub heading_deg: Option<f32>,
}

impl VehicleMarker {
    pub fn new(lat: f64, lon: f64, heading_deg: Option<f32>) -> Self {
        Self {
            position: lat_lon(lat, lon),
            heading_deg,
        }
    }
}

pub struct MapView {
    tiles: HttpTiles,
    reference_tiles: HttpTiles,
    map_memory: MapMemory,
    vehicle_marker: Option<VehicleMarker>,
    last_position: Position,
}

impl MapView {
    pub fn new(ctx: Context, restored: Option<(MapMemory, Position)>) -> Self {
        egui_extras::install_image_loaders(&ctx);
        let (map_memory, last_position) = restored.unwrap_or_else(|| {
            let mut memory = MapMemory::default();
            let _ = memory.set_zoom(3.0);
            (memory, lat_lon(0.0, 0.0))
        });
        Self {
            tiles: HttpTiles::new(EsriWorldImagery, ctx.clone()),
            reference_tiles: HttpTiles::new(EsriReferenceOverlay, ctx),
            map_memory,
            vehicle_marker: None,
            last_position,
        }
    }

    pub fn persist(&self) -> (&MapMemory, &Position) {
        (&self.map_memory, &self.last_position)
    }

    pub fn show(&mut self, ui: &mut Ui, vehicle: Option<VehicleMarker>) {
        let acquired = self.vehicle_marker.is_none() && vehicle.is_some();
        self.vehicle_marker = vehicle;
        if let Some(marker) = &self.vehicle_marker {
            self.last_position = marker.position.clone();
        }
        if acquired {
            self.map_memory.follow_my_position();
            let _ = self.map_memory.set_zoom(16.0);
        }

        let credit = self.tiles.attribution().text;

        let mut map = Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            self.last_position.clone(),
        )
        .with_layer(&mut self.reference_tiles, 1.0)
        .zoom_with_ctrl(false);

        if let Some(marker) = &self.vehicle_marker {
            map = map.with_plugin(VehiclePlugin {
                marker: marker.clone(),
            });
        }

        let response = ui.add(map);
        draw_attribution(ui, &response, credit);
    }
}

struct VehiclePlugin {
    marker: VehicleMarker,
}

impl Plugin for VehiclePlugin {
    fn run(
        self: Box<Self>,
        ui: &mut Ui,
        _response: &Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let center = projector.project(self.marker.position.clone()).to_pos2();

        match self.marker.heading_deg {
            Some(heading) => {
                let rect = egui::Rect::from_center_size(center, egui::vec2(48.0, 48.0));
                egui::Image::new(egui::include_image!("../../assets/vehicle-arrow.svg"))
                    .rotate(heading.to_radians(), egui::Vec2::splat(0.5))
                    .tint(theme::RED)
                    .paint_at(ui, rect);
            }
            None => {
                ui.painter().circle(
                    center,
                    6.0,
                    theme::RED,
                    egui::Stroke::new(1.5, egui::Color32::BLACK),
                );
            }
        }
    }
}

fn draw_attribution(ui: &Ui, response: &Response, credit: &str) {
    let galley = ui.painter().layout_no_wrap(
        credit.to_owned(),
        egui::FontId::proportional(10.0),
        egui::Color32::from_gray(230),
    );
    let min = response.rect.right_bottom() - egui::vec2(6.0, 4.0) - galley.size();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(min, galley.size()).expand(3.0),
        egui::CornerRadius::same(2),
        egui::Color32::from_black_alpha(140),
    );
    ui.painter().galley(min, galley, egui::Color32::WHITE);
}

struct EsriWorldImagery;

impl TileSource for EsriWorldImagery {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
            tile_id.zoom, tile_id.y, tile_id.x
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "© Esri, Maxar, Earthstar Geographics",
            url: "https://www.esri.com/en-us/legal/copyright-trademarks",
            logo_light: None,
            logo_dark: None,
        }
    }
}

struct EsriReferenceOverlay;

impl TileSource for EsriReferenceOverlay {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://server.arcgisonline.com/ArcGIS/rest/services/Reference/World_Boundaries_and_Places/MapServer/tile/{}/{}/{}",
            tile_id.zoom, tile_id.y, tile_id.x
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "© Esri",
            url: "https://www.esri.com/en-us/legal/copyright-trademarks",
            logo_light: None,
            logo_dark: None,
        }
    }
}
