import { RenderingInfo } from "../rerail-internal/pkg/rerail_internal";

const markerSize = 10;

export function renderMap(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  renderingInfo: RenderingInfo,
) {
  let railColors = renderingInfo.rail_colors;
  let railWidth = renderingInfo.rail_width;
  let railNumPoints = renderingInfo.rail_points_num;
  let railPointX = renderingInfo.rail_points_x;
  let railPointY = renderingInfo.rail_points_y;
  let markerX = renderingInfo.marker_points_x;
  let markerY = renderingInfo.marker_points_y;
  let stations = renderingInfo.stations;

  ctx.fillStyle = "white";
  ctx.fillRect(0, 0, width, height);

  let p = 0;
  for (let i = 0; i < railNumPoints.length; ++i) {
    let color = railColors[i];
    ctx.lineWidth = railWidth[i];
    ctx.strokeStyle = `rgb(${color.r}, ${color.g}, ${color.b})`;
    for (let j = 0; j < railNumPoints[i]; j += 2) {
      ctx.beginPath();
      ctx.moveTo(railPointX[p], railPointY[p]);
      p += 1;
      ctx.lineTo(railPointX[p], railPointY[p]);
      p += 1;
      ctx.stroke();
    }
  }

  ctx.lineWidth = 1;
  ctx.strokeStyle = "black";
  for (let i = 0; i < markerX.length; ++i) {
    ctx.strokeRect(
      markerX[i] - markerSize / 2,
      markerY[i] - markerSize / 2,
      markerSize,
      markerSize,
    );
  }

  ctx.fillStyle = "black";
  ctx.lineWidth = 1;
  ctx.font = "16px sans-serif";
  for (let i = 0; i < stations.length; ++i) {
    ctx.fillText(stations[i].name, stations[i].x, stations[i].y);
  }
}
