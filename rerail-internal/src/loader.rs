use std::io::BufRead;

use crate::railway_map::{BorderPoint, Color, Coord, RerailMap, Station};

fn next_i32<T: BufRead>(reader: &mut T) -> std::io::Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let res = ((buf[0] as i32) << 24)
        | ((buf[1] as i32) << 16)
        | ((buf[2] as i32) << 8)
        | (buf[3] as i32);
    Ok(res)
}

fn next_coord<T: BufRead>(reader: &mut T) -> std::io::Result<Coord> {
    let x = next_i32(reader)?;
    let y = next_i32(reader)?;
    Ok(Coord { x, y })
}

fn next_u8<T: BufRead>(reader: &mut T) -> std::io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn next_char<T: BufRead>(reader: &mut T) -> std::io::Result<char> {
    Ok(next_u8(reader)? as char)
}

fn next_u8_seq<T: BufRead>(reader: &mut T, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn next_sjis_string<T: BufRead>(reader: &mut T, len: usize) -> std::io::Result<String> {
    let buf = next_u8_seq(reader, len)?;
    let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&buf);
    Ok(res.into_owned())
}

fn next_sjis_string_prefixed_with_len<T: BufRead>(reader: &mut T) -> std::io::Result<String> {
    let len = next_u8(reader)? as usize;
    next_sjis_string(reader, len)
}

pub fn load_legacy_railmap_file<T: BufRead>(mut reader: &mut T) -> std::io::Result<RerailMap> {
    assert_eq!(next_char(&mut reader)?, 'R');
    assert_eq!(next_char(&mut reader)?, 'M');
    assert_eq!(next_char(&mut reader)?, 'M');
    assert_eq!(next_char(&mut reader)?, 'T');

    let _ = next_i32(&mut reader)?; // TODO: I don't know what this is
    let _initial_pos = next_coord(&mut reader)?;
    let _maybe_zoom_level = next_u8(&mut reader)?; // TODO: verify this

    assert_eq!(next_char(&mut reader)?, 'S');
    assert_eq!(next_char(&mut reader)?, 'T');

    let _station_section_size = next_i32(&mut reader)? as usize;
    let num_stations = next_i32(&mut reader)? as usize;

    let mut rerail_map = RerailMap::new();

    let mut station_indices = vec![];

    for _ in 0..num_stations {
        let station_level = (next_u8(&mut reader)? & 7) - 1;
        let _station_pos = next_coord(&mut reader)?;
        let station_name = next_sjis_string_prefixed_with_len(&mut reader)?;
        station_indices.push(rerail_map.add_station(Station::new(station_name, station_level)));
    }

    assert_eq!(next_char(&mut reader)?, 'R');
    assert_eq!(next_char(&mut reader)?, 'X');

    // rail_section_size is wrong because it assumes that each station on a railway takes 12 bytes (actually 16 bytes)
    let _rail_section_size = next_i32(&mut reader)? as usize;
    let num_rails = next_i32(&mut reader)? as usize;

    for _ in 0..num_rails {
        let rail_info = next_i32(&mut reader)?; // color (3 bytes) + rail type (1 byte) ?
        let rail_level = (rail_info & 7) as u8 - 1;
        let rail_color = Color {
            r: ((rail_info >> 24) & 255) as u8,
            g: ((rail_info >> 16) & 255) as u8,
            b: ((rail_info >> 8) & 255) as u8,
        };
        let rail_name = next_sjis_string_prefixed_with_len(&mut reader)?;
        let num_points = next_i32(&mut reader)? as usize;

        let mut points = vec![];
        for _ in 0..num_points {
            points.push(next_coord(&mut reader)?);
        }
        let mut associated_stations = vec![None; points.len()];

        let num_rail_stations = next_i32(&mut reader)? as usize;
        let mut cur_pos = 0;
        for _ in 0..num_rail_stations {
            let station_id = next_i32(&mut reader)? as usize;
            let station_pos = next_coord(&mut reader)?;
            let _ = next_i32(&mut reader)?; // TODO: I don't know what this is

            while points[cur_pos] != station_pos {
                cur_pos += 1;
            }
            assert!(associated_stations[cur_pos].is_none());
            associated_stations[cur_pos] = Some(station_indices[station_id]);
        }

        let rail_idx = rerail_map.new_railway(rail_name, rail_color, rail_level);
        for (c, st) in points.into_iter().zip(associated_stations.into_iter()) {
            if let Some(st) = &st {
                rerail_map[*st].add_railway(rail_idx);
            }
            rerail_map[rail_idx].add_point(c, st);
        }
    }

    assert_eq!(next_char(&mut reader)?, 'L');
    assert_eq!(next_char(&mut reader)?, 'S');

    let _rail_entry_section_size = next_i32(&mut reader)? as usize;
    let num_rail_entries = next_i32(&mut reader)? as usize;

    for _ in 0..num_rail_entries {
        let kind = next_u8(&mut reader)?;

        if kind == 0 {
            let _rail_id = next_i32(&mut reader)? as usize;
        } else if kind == 1 {
            let _group_name = next_sjis_string_prefixed_with_len(&mut reader)?;
        } else if kind == 2 {
            // separator
        } else {
            panic!();
        }
    }

    assert_eq!(next_char(&mut reader)?, 'B');
    assert_eq!(next_char(&mut reader)?, 'D');

    // border_section_size is wrong because it is computed assuming that an edge appears exactly once
    // in the graph, but in reality it appears twice (confusion between directed / undirected graphs)
    let _border_section_size = next_i32(&mut reader)? as usize; // TODO: I don't know what this is
    let num_border_points = next_i32(&mut reader)? as usize;

    let mut border_point_indices = vec![];
    let mut edges = vec![];
    for i in 0..num_border_points {
        let _point_level = next_u8(&mut reader)?;
        let point_coord = next_coord(&mut reader)?;

        border_point_indices.push(rerail_map.add_border_point(BorderPoint::new(point_coord)));

        let num_edges = next_u8(&mut reader)? as usize;
        for _ in 0..num_edges {
            let adj_point = next_i32(&mut reader)? as usize;
            edges.push((i, adj_point));
        }
    }

    for (u, v) in edges {
        rerail_map[border_point_indices[u]].add_neighbor(border_point_indices[v]);
        rerail_map[border_point_indices[v]].add_neighbor(border_point_indices[u]);
    }

    Ok(rerail_map)
}
