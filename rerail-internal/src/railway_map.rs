use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

pub use crate::geom::Coord;
use crate::geom::{compute_station_line_segment, Rect};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Station {
    name: String,
    railways: Vec<RailwayIndex>,
}

impl Station {
    pub fn new(name: String) -> Station {
        Station {
            name,
            railways: vec![],
        }
    }

    pub fn add_railway(&mut self, railway: RailwayIndex) -> bool {
        for i in 0..self.railways.len() {
            if self.railways[i] == railway {
                return false;
            }
        }
        self.railways.push(railway);
        true
    }
}

struct RailwayPoint {
    coord: Coord,
    station: Option<StationIndex>,
}

pub struct Railway {
    name: String,
    color: Color,
    unique_id: usize,
    points: Vec<RailwayPoint>,
}

impl Railway {
    pub fn add_point(&mut self, coord: Coord, station: Option<StationIndex>) {
        self.points.push(RailwayPoint { coord, station });
    }
}

pub struct BorderPoint {
    coord: Coord,
    neighbors: Vec<BorderPointIndex>,
}

impl BorderPoint {
    pub fn new(coord: Coord) -> BorderPoint {
        BorderPoint {
            coord,
            neighbors: vec![],
        }
    }

    pub fn add_neighbor(&mut self, neighbor: BorderPointIndex) {
        self.neighbors.push(neighbor);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StationIndex(usize);

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RailwayIndex(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorderPointIndex(usize);

#[wasm_bindgen]
pub struct RerailMap {
    stations: Vec<Option<Station>>,
    railways: Vec<Option<Railway>>,
    border_points: Vec<Option<BorderPoint>>,
    railway_unique_id_last: usize,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct StationRenderingInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

#[wasm_bindgen(getter_with_clone)]
pub struct RenderingInfo {
    pub rail_colors: Vec<Color>,
    pub rail_width: Vec<i32>,
    pub rail_points_num: Box<[i32]>,
    pub rail_points_x: Box<[i32]>,
    pub rail_points_y: Box<[i32]>,
    pub marker_points_x: Vec<i32>,
    pub marker_points_y: Vec<i32>,
    pub stations: Vec<StationRenderingInfo>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct ViewportRailwayList {
    pub rail_names: Vec<String>,
    pub rail_ids: Vec<usize>,
}

#[wasm_bindgen(typescript_custom_section)]
const VIEWPORT: &'static str = r#"
export type Viewport = {
  leftX: number,
  topY: number,
  width: number,
  height: number,
  zoom: number,
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Viewport")]
    pub type JsViewport;
}

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    #[serde(rename = "leftX")]
    left_x: i32,
    #[serde(rename = "topY")]
    top_y: i32,
    width: i32,
    height: i32,
    zoom: i32,
}

#[wasm_bindgen]
impl RerailMap {
    pub fn new() -> RerailMap {
        RerailMap {
            stations: vec![],
            railways: vec![],
            border_points: vec![],
            railway_unique_id_last: 0,
        }
    }

    pub(crate) fn add_station(&mut self, station: Station) -> StationIndex {
        let ret = StationIndex(self.stations.len());
        self.stations.push(Some(station));
        ret
    }

    fn railway_unique_id(&mut self) -> usize {
        let ret = self.railway_unique_id_last;
        self.railway_unique_id_last += 1;
        ret
    }

    pub(crate) fn new_railway(&mut self, name: String, color: Color) -> RailwayIndex {
        let railway = Railway {
            name,
            color,
            unique_id: self.railway_unique_id(),
            points: vec![],
        };
        let ret = RailwayIndex(self.railways.len());
        self.railways.push(Some(railway));
        ret
    }

    pub(crate) fn add_border_point(&mut self, border_point: BorderPoint) -> BorderPointIndex {
        let ret = BorderPointIndex(self.border_points.len());
        self.border_points.push(Some(border_point));
        ret
    }

    pub fn load(data: &[u8]) -> RerailMap {
        let mut data = data;
        crate::loader::load_legacy_railmap_file(&mut data).unwrap()
    }

    pub fn railways_in_viewport(&self, viewport: JsViewport) -> ViewportRailwayList {
        let viewport: Viewport = serde_wasm_bindgen::from_value(viewport.into()).unwrap();

        let left_x = viewport.left_x;
        let top_y = viewport.top_y;
        let view_height = viewport.height;
        let view_width = viewport.width;
        let zoom_level = viewport.zoom;

        let right_x = left_x + view_width * zoom_level;
        let bottom_y = top_y + view_height * zoom_level;
        let viewport = Rect::new(top_y, bottom_y, left_x, right_x);

        let mut rail_names = vec![];
        let mut rail_ids = vec![];

        for i in 0..self.railways.len() {
            if let Some(railway) = &self.railways[i] {
                let mut is_displayed = false;
                for j in 1..railway.points.len() {
                    if viewport.crosses_with_line_segment(
                        railway.points[j - 1].coord,
                        railway.points[j].coord,
                    ) {
                        is_displayed = true;
                        break;
                    }
                }

                if is_displayed {
                    rail_names.push(railway.name.clone());
                    rail_ids.push(railway.unique_id);
                }
            }
        }

        ViewportRailwayList {
            rail_names,
            rail_ids,
        }
    }

    pub fn render(&self, viewport: JsViewport, selected_rail_id: Option<usize>) -> RenderingInfo {
        let viewport: Viewport = serde_wasm_bindgen::from_value(viewport.into()).unwrap();

        let left_x = viewport.left_x;
        let top_y = viewport.top_y;
        let view_height = viewport.height;
        let view_width = viewport.width;
        let zoom_level = viewport.zoom;

        let right_x = left_x + view_width * zoom_level;
        let bottom_y = top_y + view_height * zoom_level;
        let viewport = Rect::new(top_y, bottom_y, left_x, right_x);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points_x = vec![];
        let mut rail_points_y = vec![];
        let mut stations = vec![];

        let mut marker_points_x = vec![];
        let mut marker_points_y = vec![];

        for railway in &self.railways {
            if let Some(railway) = railway {
                if Some(railway.unique_id) == selected_rail_id {
                    for i in 0..railway.points.len() {
                        let c = railway.points[i].coord;
                        if viewport.contains(c) {
                            marker_points_x.push((c.x - left_x) / zoom_level);
                            marker_points_y.push((c.y - top_y) / zoom_level);
                        }
                    }
                }

                let mut num = 0;
                for i in 1..railway.points.len() {
                    if viewport.crosses_with_line_segment(
                        railway.points[i - 1].coord,
                        railway.points[i].coord,
                    ) {
                        num += 2;

                        let c0 = railway.points[i - 1].coord;
                        let c1 = railway.points[i].coord;

                        rail_points_x.push((c0.x - left_x) / zoom_level);
                        rail_points_x.push((c1.x - left_x) / zoom_level);
                        rail_points_y.push((c0.y - top_y) / zoom_level);
                        rail_points_y.push((c1.y - top_y) / zoom_level);
                    }
                }

                if num > 0 {
                    rail_colors.push(railway.color);
                    rail_width.push(1);
                    rail_points_num.push(num);
                }
            }
        }

        let mut station_points_x = vec![];
        let mut station_points_y = vec![];
        let mut station_rendered = std::collections::BTreeSet::<StationIndex>::new();

        for railway in &self.railways {
            if let Some(railway) = railway {
                for i in 0..railway.points.len() {
                    if let Some(station_idx) = railway.points[i].station {
                        if !railway.points[i].station.is_some() {
                            continue;
                        }
                        if !viewport.contains(railway.points[i].coord) {
                            continue;
                        }

                        let prev = if i == 0 {
                            None
                        } else {
                            Some(railway.points[i - 1].coord)
                        };
                        let next = if i + 1 == railway.points.len() {
                            None
                        } else {
                            Some(railway.points[i + 1].coord)
                        };

                        let (c0, c1) =
                            compute_station_line_segment(prev, railway.points[i].coord, next, 200);

                        station_points_x.push((c0.x - left_x) / zoom_level);
                        station_points_x.push((c1.x - left_x) / zoom_level);
                        station_points_y.push((c0.y - top_y) / zoom_level);
                        station_points_y.push((c1.y - top_y) / zoom_level);

                        if station_rendered.contains(&station_idx) {
                            continue;
                        }
                        station_rendered.insert(station_idx);

                        let station = &self[station_idx];
                        stations.push(StationRenderingInfo {
                            name: station.name.clone(),
                            x: (railway.points[i].coord.x - left_x) / zoom_level,
                            y: (railway.points[i].coord.y - top_y) / zoom_level,
                        });
                    }
                }
            }
        }

        if station_points_x.len() > 0 {
            rail_colors.push(Color {
                r: 148,
                g: 148,
                b: 148,
            });
            rail_width.push(4);
            rail_points_num.push(station_points_x.len() as i32);
            rail_points_x.extend(station_points_x);
            rail_points_y.extend(station_points_y);
        }

        let mut border_points_x = vec![];
        let mut border_points_y = vec![];

        for i in 0..self.border_points.len() {
            if let Some(pt) = &self.border_points[i] {
                for &j in &pt.neighbors {
                    if i < j.0 {
                        let pt2 = &self[j];
                        if viewport.crosses_with_line_segment(pt.coord, pt2.coord) {
                            border_points_x.push((pt.coord.x - left_x) / zoom_level);
                            border_points_y.push((pt.coord.y - top_y) / zoom_level);
                            border_points_x.push((pt2.coord.x - left_x) / zoom_level);
                            border_points_y.push((pt2.coord.y - top_y) / zoom_level);
                        }
                    }
                }
            }
        }

        if border_points_x.len() > 0 {
            rail_colors.push(Color { r: 0, g: 0, b: 0 });
            rail_width.push(1);
            rail_points_num.push(border_points_x.len() as i32);
            rail_points_x.extend(border_points_x);
            rail_points_y.extend(border_points_y);
        }

        RenderingInfo {
            rail_colors,
            rail_width,
            rail_points_num: rail_points_num.into_boxed_slice(),
            rail_points_x: rail_points_x.into_boxed_slice(),
            rail_points_y: rail_points_y.into_boxed_slice(),
            marker_points_x,
            marker_points_y,
            stations,
        }
    }
}

impl Index<StationIndex> for RerailMap {
    type Output = Station;

    fn index(&self, index: StationIndex) -> &Self::Output {
        self.stations[index.0].as_ref().unwrap()
    }
}

impl Index<RailwayIndex> for RerailMap {
    type Output = Railway;

    fn index(&self, index: RailwayIndex) -> &Self::Output {
        self.railways[index.0].as_ref().unwrap()
    }
}

impl Index<BorderPointIndex> for RerailMap {
    type Output = BorderPoint;

    fn index(&self, index: BorderPointIndex) -> &Self::Output {
        self.border_points[index.0].as_ref().unwrap()
    }
}

impl IndexMut<StationIndex> for RerailMap {
    fn index_mut(&mut self, index: StationIndex) -> &mut Self::Output {
        self.stations[index.0].as_mut().unwrap()
    }
}

impl IndexMut<RailwayIndex> for RerailMap {
    fn index_mut(&mut self, index: RailwayIndex) -> &mut Self::Output {
        self.railways[index.0].as_mut().unwrap()
    }
}

impl IndexMut<BorderPointIndex> for RerailMap {
    fn index_mut(&mut self, index: BorderPointIndex) -> &mut Self::Output {
        self.border_points[index.0].as_mut().unwrap()
    }
}
