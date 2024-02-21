use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

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
pub struct ViewportSpec {
    #[serde(rename = "leftX")]
    left_x: i32,
    #[serde(rename = "topY")]
    top_y: i32,
    width: i32,
    height: i32,
    zoom: i32,
}

#[derive(Clone, Copy)]
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
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct NearestSegment {
    pub idx0: usize,
    pub idx1: Option<usize>,
}

#[wasm_bindgen]
impl NearestSegment {
    pub fn clone(&self) -> NearestSegment {
        Clone::clone(&self)
    }
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
        self.railways[railway_id].as_mut().unwrap().points.insert(
            i,
            RailwayPoint {
                coord: Coord::new(x, y),
                station: None,
            },
        )
    }

    pub fn move_railway_point(&mut self, railway_id: usize, i: usize, x: i32, y: i32) {
        self.railways[railway_id].as_mut().unwrap().points[i].coord = Coord::new(x, y);
    }

    pub fn railways_in_viewport(&self, viewport: JsViewport) -> ViewportRailwayList {
        let viewport: ViewportSpec = serde_wasm_bindgen::from_value(viewport.into()).unwrap();
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

    pub fn render(
        &self,
        viewport: JsViewport,
        selected_rail_id: Option<usize>,
        skip_nearest_segment: Option<NearestSegment>,
        mouse_x: Option<i32>,
        mouse_y: Option<i32>,
    ) -> RenderingInfo {
        let viewport: ViewportSpec = serde_wasm_bindgen::from_value(viewport.into()).unwrap();
        let viewport = Viewport::new(viewport);

        let mut rail_colors = vec![];
        let mut rail_width = vec![];
        let mut rail_points_num = vec![];
        let mut rail_points = vec![];
        let mut stations = vec![];

        let mut marker_points = vec![];

        for railway in &self.railways {
            if let Some(railway) = railway {
                let mut num = 0;

                if Some(railway.unique_id) == selected_rail_id {
                    if let (Some(skip_nearest_segment), Some(mouse_x), Some(mouse_y)) =
                        (&skip_nearest_segment, mouse_x, mouse_y)
                    {
                        let mouse = PhysicalCoord {
                            x: mouse_x,
                            y: mouse_y,
                        };
                        marker_points.push(mouse);
                        let idx0 = skip_nearest_segment.idx0;
                        if let Some(idx1) = skip_nearest_segment.idx1 {
                            num += 4;
                            rail_points.push(mouse);
                            rail_points
                                .push(viewport.to_physical_point(railway.points[idx0].coord));
                            rail_points.push(mouse);
                            rail_points
                                .push(viewport.to_physical_point(railway.points[idx1].coord));
                        } else {
                            if idx0 > 0 {
                                num += 2;
                                rail_points.push(mouse);
                                rail_points.push(
                                    viewport.to_physical_point(railway.points[idx0 - 1].coord),
                                );
                            }
                            if idx0 + 1 < railway.points.len() {
                                num += 2;
                                rail_points.push(mouse);
                                rail_points.push(
                                    viewport.to_physical_point(railway.points[idx0 + 1].coord),
                                );
                            }
                        }
                    }

                    for i in 0..railway.points.len() {
                        if let Some(skip_nearest_segment) = &skip_nearest_segment {
                            if skip_nearest_segment.idx0 == i && skip_nearest_segment.idx1.is_none()
                            {
                                continue;
                            }
                        }

                        let c = railway.points[i].coord;
                        if viewport.contains(c) {
                            marker_points.push(viewport.to_physical_point(c));
                        }
                    }
                }

                for i in 1..railway.points.len() {
                    if Some(railway.unique_id) == selected_rail_id {
                        if let Some(skip_nearest_segment) = &skip_nearest_segment {
                            if skip_nearest_segment.idx0 == i - 1
                                && skip_nearest_segment.idx1 == Some(i)
                            {
                                continue;
                            }
                            if (skip_nearest_segment.idx0 == i - 1
                                || skip_nearest_segment.idx0 == i)
                                && skip_nearest_segment.idx1.is_none()
                            {
                                continue;
                            }
                        }
                    }

                    if viewport.crosses_with_line_segment(
                        railway.points[i - 1].coord,
                        railway.points[i].coord,
                    ) {
                        num += 2;

                        rail_points.push(viewport.to_physical_point(railway.points[i - 1].coord));
                        rail_points.push(viewport.to_physical_point(railway.points[i].coord));
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

                        station_points.push(viewport.to_physical_point(c0));
                        station_points.push(viewport.to_physical_point(c1));

                        if station_rendered.contains(&station_idx) {
                            continue;
                        }
                        station_rendered.insert(station_idx);

                        let station = &self[station_idx];
                        let pt = viewport.to_physical_point(railway.points[i].coord);
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
        viewport: JsViewport,
        rail_id: usize,
        x: i32,
        y: i32,
        max_dist: i32,
    ) -> Option<NearestSegment> {
        let viewport: ViewportSpec = serde_wasm_bindgen::from_value(viewport.into()).unwrap();
        let viewport = Viewport::new(viewport);
        let p = Coord::new(x, y);

        let threshold = max_dist as i64 * max_dist as i64;

        for railway in &self.railways {
            if let Some(railway) = railway {
                if railway.unique_id != rail_id {
                    continue;
                }

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
                        idx0: nearest.1,
                        idx1: None,
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
                    return Some(NearestSegment {
                        idx0: nearest.1 - 1,
                        idx1: Some(nearest.1),
                    });
                } else {
                    return None;
                }
            }
        }
        None
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
