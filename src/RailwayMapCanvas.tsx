import { useState, useRef, useEffect } from 'react';
import { RerailMap } from '../rerail-internal/pkg/rerail_internal'
import { renderMap } from './renderer'

type RailwayMapCanvasProps = {
    height: number,
    width: number,
    topX: number,
    topY: number,
    zoomLevel: number,
    setViewport: (x: number, y: number, zoomLevel: number) => void,
    railwayMap: RerailMap | null,
};

type ClickInfo = {
    mouseX: number,
    mouseY: number,
    topX: number,
    topY: number,
};

const zoomLevels = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000];

export const RailwayMapCanvas = (props: RailwayMapCanvasProps) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const [clickPos, setClickPos] = useState<ClickInfo | null>(null);

    useEffect(
        () => {
            if (!props.railwayMap) return;
            const railwayMap = props.railwayMap;
            const renderInfo = railwayMap.render(props.topX, props.topY, props.height, props.width, zoomLevels[props.zoomLevel]);

            const canvas = canvasRef.current!;
            const ctx = canvas.getContext("2d")!;

            renderMap(ctx, props.width, props.height, renderInfo);
        },
        [props]
    );

    const mouseDownHandler = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
        const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
        const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

        setClickPos({mouseX: x, mouseY: y, topX: props.topX, topY: props.topY});
    };

    const mouseMoveHandler = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
        if (clickPos === null) return;

        const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
        const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

        const zoom = zoomLevels[props.zoomLevel];
        const newTopX = clickPos.topX + (clickPos.mouseX - x) * zoom;
        const newTopY = clickPos.topY + (clickPos.mouseY - y) * zoom;
        props.setViewport(newTopX, newTopY, props.zoomLevel);
    };

    const mouseUpHandler = () => setClickPos(null);

    const wheelHandler = (e: React.WheelEvent) => {
        if (clickPos !== null) return;

        const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
        const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

        const zoomLevel = props.zoomLevel;
        const newZoomLevel = zoomLevel + (e.deltaY < 0 ? -1 : 1);

        if (!(0 <= newZoomLevel && newZoomLevel < zoomLevels.length)) return;

        const newTopX = props.topX + x * (zoomLevels[zoomLevel] - zoomLevels[newZoomLevel]);
        const newTopY = props.topY + y * (zoomLevels[zoomLevel] - zoomLevels[newZoomLevel]);
        props.setViewport(newTopX, newTopY, newZoomLevel);
    };

    return (
        <canvas
            height={props.height}
            width={props.width}
            ref={canvasRef}
            onMouseDown={mouseDownHandler}
            onMouseMove={mouseMoveHandler}
            onMouseUp={mouseUpHandler}
            onMouseOut={mouseUpHandler}
            onWheel={wheelHandler}
            style={{verticalAlign: "top", cursor: clickPos !== null ? "move" : "auto"}}
        />
    );
};
