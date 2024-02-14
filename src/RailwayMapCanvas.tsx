import { useState, useRef, useEffect } from 'react';
import { RailwayMap } from '../rerail-internal/pkg/rerail_internal'
import { renderMap } from './renderer'

type RailwayMapCanvasProps = {
    height: number,
    width: number,
    topX: number,
    topY: number,
    zoom: number,
    setTopPos: (x: number, y: number) => void,
    railwayMap: RailwayMap | null,
};

type ClickInfo = {
    mouseX: number,
    mouseY: number,
    topX: number,
    topY: number,
};

export const RailwayMapCanvas = (props: RailwayMapCanvasProps) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const [clickPos, setClickPos] = useState<ClickInfo | null>(null);

    useEffect(
        () => {
            if (!props.railwayMap) return;
            const railwayMap = props.railwayMap;
            const renderInfo = railwayMap.render(props.topX, props.topY, props.height, props.width, props.zoom);

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

        const newTopX = clickPos.topX + (clickPos.mouseX - x) * 50;
        const newTopY = clickPos.topY + (clickPos.mouseY - y) * 50;
        props.setTopPos(newTopX, newTopY);
    };

    const mouseUpHandler = () => setClickPos(null);

    return (
        <canvas
            height={props.height}
            width={props.width}
            ref={canvasRef}
            onMouseDown={mouseDownHandler}
            onMouseMove={mouseMoveHandler}
            onMouseUp={mouseUpHandler}
            onMouseOut={mouseUpHandler}
        />
    );
};
