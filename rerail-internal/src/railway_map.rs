use wasm_bindgen::prelude::*;

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
    points: Vec<RailwayPoint>,
}

impl Railway {
    pub fn new(name: String, color: Color) -> Railway {
        Railway {
            name,
            color,
            points: vec![],
        }
    }

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
    pub stations: Vec<StationRenderingInfo>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct ViewportRailwayList {
    pub rail_names: Vec<String>,
    pub rail_indices: Vec<RailwayIndex>, // TODO: use persistent index
}

#[wasm_bindgen]
impl RerailMap {
    pub fn new() -> RerailMap {
        RerailMap {
            stations: vec![],
            railways: vec![],
            border_points: vec![],
        }
    }

    pub(crate) fn add_station(&mut self, station: Station) -> StationIndex {
        let ret = StationIndex(self.stations.len());
        self.stations.push(Some(station));
        ret
    }

    pub(crate) fn add_railway(&mut self, railway: Railway) -> RailwayIndex {
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

    pub fn railways_in_viewport(
        &self,
        left_x: i32,
        top_y: i32,
        view_height: i32,
        view_width: i32,
        zoom_level: i32,
    ) -> ViewportRailwayList {
        let right_x = left_x + view_width * zoom_level;
        let bottom_y = top_y + view_height * zoom_level;
        let viewport = Rect::new(top_y, bottom_y, left_x, right_x);

        let mut rail_names = vec![];
        let mut rail_indices = vec![];

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
                    rail_indices.push(RailwayIndex(i));
                }
            }
        }

        ViewportRailwayList {
            rail_names,
            rail_indices,
        }
    }

    pub fn render(
        &self,
        left_x: i32,
        top_y: i32,
        view_height: i32,
        view_width: i32,
        zoom_level: i32,
    ) -> RenderingInfo {
        let right_x = left_x + view_width * zoom_level;
        let bottom_y = top_y + view_height * zoom_level;
        let viewport = Rect::new(top_y, bottom_y, left_x, right_x);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points_x = vec![];
        let mut rail_points_y = vec![];
        let mut stations = vec![];

        for railway in &self.railways {
            if let Some(railway) = railway {
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
