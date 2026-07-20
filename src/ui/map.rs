use egui::{Context, Ui};
use walkers::{HttpTiles, Map, MapMemory, Position, TileId, Tiles, sources::{TileSource, Attribution}};

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

pub fn imagery_tiles(ctx: Context) -> HttpTiles {
    HttpTiles::new(EsriWorldImagery, ctx)
}

pub fn reference_tiles(ctx: Context) -> HttpTiles {
    HttpTiles::new(EsriReferenceOverlay, ctx)
}

pub fn show(
    ui: &mut Ui,
    tiles: &mut HttpTiles,
    reference: &mut HttpTiles,
    memory: &mut MapMemory,
    center: Position,
) {
    let credit = tiles.attribution().text;
    let response = ui.add(
        Map::new(Some(tiles), memory, center)
            .with_layer(reference, 1.0)
            .zoom_with_ctrl(false),
    );
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
