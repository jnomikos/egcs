use super::theme;
use egui::{Context, Response, Ui};
use walkers::{
    HttpTiles, Map, MapMemory, Plugin, Position, Projector, TileId, Tiles, lat_lon,
    sources::{Attribution, TileSource},
};

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

pub enum MapAction {
    Goto(Position),
}

struct GotoPrompt {
    position: Position,
    screen_pos: egui::Pos2,
}

pub struct MapView {
    tiles: HttpTiles,
    reference_tiles: HttpTiles,
    map_memory: MapMemory,
    vehicle_marker: Option<VehicleMarker>,
    last_position: Position,
    goto_prompt: Option<GotoPrompt>,
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
            goto_prompt: None,
        }
    }

    pub fn persist(&self) -> (&MapMemory, &Position) {
        (&self.map_memory, &self.last_position)
    }

    pub fn show(&mut self, ui: &mut Ui, vehicle: Option<VehicleMarker>) -> Option<MapAction> {
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

        if self.vehicle_marker.is_none() {
            self.goto_prompt = None;
        }

        let mut map = Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            self.last_position.clone(),
        )
        .with_layer(&mut self.reference_tiles, 1.0)
        .zoom_with_ctrl(false);

        if let Some(marker) = &self.vehicle_marker {
            map = map
                .with_plugin(VehiclePlugin {
                    marker: marker.clone(),
                })
                .with_plugin(ClickPlugin {
                    prompt: &mut self.goto_prompt,
                });
        }

        let response = ui.add(map);
        draw_attribution(ui, &response, credit);
        self.goto_popup(ui)
    }

    fn goto_popup(&mut self, ui: &Ui) -> Option<MapAction> {
        let prompt = self.goto_prompt.as_ref()?;
        let mut action = None;
        let mut dismiss = false;

        egui::Area::new(egui::Id::new("map_goto_prompt"))
            .fixed_pos(prompt.screen_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Fly here").clicked() {
                            action = Some(MapAction::Goto(prompt.position.clone()));
                            dismiss = true;
                        }
                        if ui.button("Cancel").clicked() {
                            dismiss = true;
                        }
                    });
                });
            });

        if dismiss {
            self.goto_prompt = None;
        }
        action
    }
}

struct ClickPlugin<'a> {
    prompt: &'a mut Option<GotoPrompt>,
}

impl Plugin for ClickPlugin<'_> {
    fn run(
        self: Box<Self>,
        _ui: &mut Ui,
        response: &Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                *self.prompt = Some(GotoPrompt {
                    position: projector.unproject(pos.to_vec2()),
                    screen_pos: pos,
                });
            }
        }

        if let Some(prompt) = self.prompt.as_mut() {
            prompt.screen_pos = projector.project(prompt.position.clone()).to_pos2();
        }
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
                    egui::Stroke::new(1.5_f32, egui::Color32::BLACK),
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
