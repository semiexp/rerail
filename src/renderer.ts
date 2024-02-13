import { RenderingInfo } from '../rerail-internal/pkg/rerail_internal'

export function renderMap(ctx: CanvasRenderingContext2D, renderingInfo: RenderingInfo) {
    let railColors = renderingInfo.rail_colors;
    let railNumPoints = renderingInfo.rail_points_num;
    let railPointX = renderingInfo.rail_points_x;
    let railPointY = renderingInfo.rail_points_y;
    console.log(railColors);

    let p = 0;
    for (let i = 0; i < railNumPoints.length; ++i) {
        let color = railColors[i];
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
}
