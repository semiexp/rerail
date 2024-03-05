use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};
use tsify::Tsify;

pub use crate::geom::Coord;
use crate::geom::{
    compute_station_line_segment, distance_norm_square_point_line_segment,
    distance_norm_square_points, Rect,
};

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

#[derive(Clone)]
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

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ViewportSpec {
    #[serde(rename = "leftX")]
    left_x: i32,
    #[serde(rename = "topY")]
    top_y: i32,
    width: i32,
    height: i32,
    zoom: i32,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RenderingOptions {
    #[tsify(optional)]
    #[serde(rename = "selectedRailId")]
    selected_rail_id: Option<usize>,
    #[tsify(optional)]
    #[serde(rename = "skipNearestSegment")]
    skip_nearest_segment: Option<NearestSegment>,
    #[tsify(optional)]
    mouse: Option<PhysicalCoord>,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct StationInfo {
    name: String,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct PhysicalCoord {
    x: i32,
    y: i32,
}

impl PhysicalCoord {
    fn as_coord(&self) -> Coord {
        Coord {
            x: self.x,
            y: self.y,
        }
    }
}

fn split_into_x_and_y(points: &[PhysicalCoord]) -> (Vec<i32>, Vec<i32>) {
    let mut xs = vec![];
    let mut ys = vec![];
    for pt in points {
        xs.push(pt.x);
        ys.push(pt.y);
    }
    (xs, ys)
}

struct Viewport {
    left_x: i32,
    top_y: i32,
    zoom: i32,
    bounding_box: Rect,
}

impl Viewport {
    fn new(spec: ViewportSpec) -> Viewport {
        let bottom = spec.top_y + spec.height * spec.zoom;
        let right = spec.left_x + spec.width * spec.zoom;

        Viewport {
            left_x: spec.left_x,
            top_y: spec.top_y,
            zoom: spec.zoom,
            bounding_box: Rect::new(spec.top_y, bottom, spec.left_x, right),
        }
    }

    fn contains(&self, coord: Coord) -> bool {
        self.bounding_box.contains(coord)
    }

    fn crosses_with_line_segment(&self, a: Coord, b: Coord) -> bool {
        self.bounding_box.crosses_with_line_segment(a, b)
    }

    fn to_physical_point(&self, coord: Coord) -> PhysicalCoord {
        PhysicalCoord {
            x: (coord.x - self.left_x) / self.zoom,
            y: (coord.y - self.top_y) / self.zoom,
        }
    }

    fn from_physical_point(&self, coord: PhysicalCoord) -> Coord {
        Coord {
            x: coord.x * self.zoom + self.left_x,
            y: coord.y * self.zoom + self.top_y,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct NearestSegment {
    index: usize,
    #[serde(rename = "betweenPoints")]
    between_points: bool,
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

    pub fn insert_railway_point(&mut self, railway_id: usize, i: usize, x: i32, y: i32) {
        let railway =
            RerailMap::find_railway_by_unique_id_mut(&mut self.railways, railway_id).unwrap();
        railway.points.insert(
            i,
            RailwayPoint {
                coord: Coord::new(x, y),
                station: None,
            },
        )
    }

    pub fn move_railway_point(&mut self, railway_id: usize, i: usize, x: i32, y: i32) {
        let railway =
            RerailMap::find_railway_by_unique_id_mut(&mut self.railways, railway_id).unwrap();
        railway.points[i].coord = Coord::new(x, y);
    }

    pub fn railways_in_viewport(&self, viewport: ViewportSpec) -> ViewportRailwayList {
        let viewport = Viewport::new(viewport);

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

    pub fn render(&self, viewport: ViewportSpec, opts: RenderingOptions) -> RenderingInfo {
        let viewport = Viewport::new(viewport);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points = vec![];
        let mut stations = vec![];

        let mut marker_points = vec![];

        let mut selected_railway_points = vec![];
        if let Some(id) = opts.selected_rail_id {
            if let Some(selected_railway) = RerailMap::find_railway_by_unique_id(&self.railways, id)
            {
                selected_railway_points = selected_railway.points.clone();

                if let (Some(skip_nearest_segment), Some(mouse_coord)) =
                    (&opts.skip_nearest_segment, opts.mouse)
                {
                    let mouse_coord = viewport.from_physical_point(mouse_coord);
                    if skip_nearest_segment.between_points {
                        selected_railway_points.insert(
                            skip_nearest_segment.index + 1,
                            RailwayPoint {
                                coord: mouse_coord,
                                station: None,
                            },
                        );
                    } else {
                        selected_railway_points[skip_nearest_segment.index].coord = mouse_coord;
                    }
                }
            }
        }

        for pt in &selected_railway_points {
            let coord = pt.coord;
            if viewport.contains(coord) {
                marker_points.push(viewport.to_physical_point(coord))
            }
        }

        for railway in &self.railways {
            if let Some(railway) = railway {
                let railway_points = if Some(railway.unique_id) == opts.selected_rail_id {
                    &selected_railway_points
                } else {
                    &railway.points
                };

                let mut num = 0;
                for i in 1..railway_points.len() {
                    if viewport.crosses_with_line_segment(
                        railway_points[i - 1].coord,
                        railway_points[i].coord,
                    ) {
                        num += 2;

                        rail_points.push(viewport.to_physical_point(railway_points[i - 1].coord));
                        rail_points.push(viewport.to_physical_point(railway_points[i].coord));
                    }
                }

                if num > 0 {
                    rail_colors.push(railway.color);
                    rail_width.push(1);
                    rail_points_num.push(num);
                }
            }
        }

        let mut station_points = vec![];
        let mut station_rendered = std::collections::BTreeSet::<StationIndex>::new();

        for railway in &self.railways {
            if let Some(railway) = railway {
                let railway_points = if Some(railway.unique_id) == opts.selected_rail_id {
                    &selected_railway_points
                } else {
                    &railway.points
                };
                for i in 0..railway_points.len() {
                    if let Some(station_idx) = railway_points[i].station {
                        if !railway_points[i].station.is_some() {
                            continue;
                        }
                        if !viewport.contains(railway_points[i].coord) {
                            continue;
                        }

                        let prev = if i == 0 {
                            None
                        } else {
                            Some(railway_points[i - 1].coord)
                        };
                        let cur = railway_points[i].coord;
                        let next = if i + 1 == railway_points.len() {
                            None
                        } else {
                            Some(railway_points[i + 1].coord)
                        };

                        let (c0, c1) = compute_station_line_segment(prev, cur, next, 200);

                        station_points.push(viewport.to_physical_point(c0));
                        station_points.push(viewport.to_physical_point(c1));

                        if station_rendered.contains(&station_idx) {
                            continue;
                        }
                        station_rendered.insert(station_idx);

                        let station = &self[station_idx];
                        let pt = viewport.to_physical_point(railway_points[i].coord);
                        stations.push(StationRenderingInfo {
                            name: station.name.clone(),
                            x: pt.x,
                            y: pt.y,
                        });
                    }
                }
            }
        }

        if station_points.len() > 0 {
            rail_colors.push(Color {
                r: 148,
                g: 148,
                b: 148,
            });
            rail_width.push(4);
            rail_points_num.push(station_points.len() as i32);
            rail_points.extend(station_points);
        }

        let mut border_points = vec![];

        for i in 0..self.border_points.len() {
            if let Some(pt) = &self.border_points[i] {
                for &j in &pt.neighbors {
                    if i < j.0 {
                        let pt2 = &self[j];
                        if viewport.crosses_with_line_segment(pt.coord, pt2.coord) {
                            border_points.push(viewport.to_physical_point(pt.coord));
                            border_points.push(viewport.to_physical_point(pt2.coord));
                        }
                    }
                }
            }
        }

        if border_points.len() > 0 {
            rail_colors.push(Color { r: 0, g: 0, b: 0 });
            rail_width.push(1);
            rail_points_num.push(border_points.len() as i32);
            rail_points.extend(border_points);
        }

        let (rail_points_x, rail_points_y) = split_into_x_and_y(&rail_points);
        let (marker_points_x, marker_points_y) = split_into_x_and_y(&marker_points);

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

    pub fn find_nearest_segment(
        &self,
        viewport: ViewportSpec,
        rail_id: usize,
        x: i32,
        y: i32,
        max_dist: i32,
    ) -> Option<NearestSegment> {
        let viewport = Viewport::new(viewport);
        let p = Coord::new(x, y);

        let threshold = max_dist as i64 * max_dist as i64;

        let railway = RerailMap::find_railway_by_unique_id(&self.railways, rail_id)?;

        let mut nearest = (threshold + 1, 0);

        for i in 0..railway.points.len() {
            let d = distance_norm_square_points(
                viewport
                    .to_physical_point(railway.points[i].coord)
                    .as_coord(),
                p,
            );
            if d < nearest.0 {
                nearest = (d, i);
            }
        }

        if nearest.0 <= threshold {
            return Some(NearestSegment {
                index: nearest.1,
                between_points: false,
            });
        }

        let mut nearest = (threshold + 1, 0);

        for i in 1..railway.points.len() {
            let c0 = viewport
                .to_physical_point(railway.points[i - 1].coord)
                .as_coord();
            let c1 = viewport
                .to_physical_point(railway.points[i].coord)
                .as_coord();
            let d = distance_norm_square_point_line_segment(c0, c1, p);
            if d < nearest.0 {
                nearest = (d, i);
            }
        }

        if nearest.0 <= threshold {
            Some(NearestSegment {
                index: nearest.1 - 1,
                between_points: true,
            })
        } else {
            None
        }
    }

    pub fn get_station_info(&self, rail_id: usize, point_idx: usize) -> Option<StationInfo> {
        let railway = RerailMap::find_railway_by_unique_id(&self.railways, rail_id);
        if let Some(railway) = railway {
            if let Some(station_idx) = railway.points[point_idx].station {
                let station = &self[station_idx];
                return Some(StationInfo {
                    name: station.name.clone(),
                });
            }
        }
        None
    }

    pub fn set_station_info(&mut self, rail_id: usize, point_idx: usize, info: StationInfo) {
        let railway = RerailMap::find_railway_by_unique_id_mut(&mut self.railways, rail_id);
        if let Some(railway) = railway {
            if let Some(station_idx) = railway.points[point_idx].station {
                self[station_idx].name = info.name;
            } else {
                let station_idx = StationIndex(self.stations.len());
                self.stations.push(Some(Station::new(info.name)));
                railway.points[point_idx].station = Some(station_idx);
                self[station_idx].add_railway(RailwayIndex(rail_id));
            }
        }
    }

    fn find_railway_by_unique_id<'a>(
        railways: &'a [Option<Railway>],
        unique_id: usize,
    ) -> Option<&'a Railway> {
        railways.iter().find_map(|railway| {
            if let Some(railway) = railway {
                if railway.unique_id == unique_id {
                    Some(railway)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn find_railway_by_unique_id_mut<'a>(
        railways: &'a mut [Option<Railway>],
        unique_id: usize,
    ) -> Option<&'a mut Railway> {
        railways.iter_mut().find_map(|railway| {
            if let Some(railway) = railway {
                if railway.unique_id == unique_id {
                    Some(railway)
                } else {
                    None
                }
            } else {
                None
            }
        })
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
