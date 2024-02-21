import init_wasm, {
  ViewportRailwayList,
} from "../rerail-internal/pkg/rerail_internal";
import {
  RerailMap as InternalRerailMap,
  Viewport,
  NearestSegment,
  RenderingInfo,
} from "../rerail-internal/pkg/rerail_internal";
export type {
  Viewport,
  ViewportRailwayList,
  NearestSegment,
  RenderingInfo,
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

  render(
    viewport: Viewport,
    selectedRailId?: number,
    skipNearestSegment?: NearestSegment,
    mouseX?: number,
    mouseY?: number,
  ): RenderingInfo {
    return this.data.render(
      viewport,
      selectedRailId,
      skipNearestSegment,
      mouseX,
      mouseY,
    );
  }

  railwaysInViewport(viewport: Viewport): ViewportRailwayList {
    return this.data.railways_in_viewport(viewport);
  }

  findNearestSegment(
    viewport: Viewport,
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
}
