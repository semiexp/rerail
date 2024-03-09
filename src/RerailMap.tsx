import init_wasm, {
  RailwayInfo,
  StationInfo,
  ViewportRailwayList,
} from "../rerail-internal/pkg/rerail_internal";
import {
  RerailMap as InternalRerailMap,
  ViewportSpec,
  NearestSegment,
  RenderingInfo,
  RenderingOptions,
} from "../rerail-internal/pkg/rerail_internal";
export type {
  ViewportSpec,
  ViewportRailwayList,
  NearestSegment,
  RailwayInfo,
  RenderingInfo,
  RenderingOptions,
  StationInfo,
} from "../rerail-internal/pkg/rerail_internal";

await init_wasm();

export class RerailMap {
  _data?: InternalRerailMap;

  constructor(data: InternalRerailMap) {
    this._data = data;
  }

  static load(buffer: Uint8Array): RerailMap {
    const data = InternalRerailMap.load(buffer);
    return new RerailMap(data);
  }

  private get data(): InternalRerailMap {
    if (!this._data) {
      throw new Error();
    }
    return this._data;
  }

  render(viewport: ViewportSpec, opts: RenderingOptions): RenderingInfo {
    return this.data.render(viewport, opts);
  }

  railwaysInViewport(viewport: ViewportSpec): ViewportRailwayList {
    return this.data.railways_in_viewport(viewport);
  }

  findNearestSegment(
    viewport: ViewportSpec,
    railId: number,
    x: number,
    y: number,
    maxDist: number,
  ): NearestSegment | undefined {
    return this.data.find_nearest_segment(viewport, railId, x, y, maxDist);
  }

  insertRailwayPoint(
    railwayId: number,
    i: number,
    x: number,
    y: number,
  ): RerailMap {
    let data = this.data;
    this._data = undefined;
    data.insert_railway_point(railwayId, i, x, y);
    return new RerailMap(data);
  }

  moveRailwayPoint(
    railwayId: number,
    i: number,
    x: number,
    y: number,
  ): RerailMap {
    let data = this.data;
    this._data = undefined;
    data.move_railway_point(railwayId, i, x, y);
    return new RerailMap(data);
  }

  getStationInfo(railwayId: number, pointIdx: number): StationInfo | undefined {
    return this.data.get_station_info(railwayId, pointIdx);
  }

  setStationInfo(
    railwayId: number,
    pointIdx: number,
    stationInfo: StationInfo,
  ): RerailMap {
    let data = this.data;
    this._data = undefined;
    data.set_station_info(railwayId, pointIdx, stationInfo);
    return new RerailMap(data);
  }

  getRailwayInfo(railwayId: number): RailwayInfo {
    return this.data.get_railway_info(railwayId);
  }

  setRailwayInfo(railwayId: number, info: RailwayInfo): RerailMap {
    let data = this.data;
    this._data = undefined;
    data.set_railway_info(railwayId, info);
    return new RerailMap(data);
  }
}
