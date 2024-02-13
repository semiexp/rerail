import { ChangeEvent, useRef } from 'react';
import { RailwayMap } from '../rerail-internal/pkg/rerail_internal'
import { renderMap } from './renderer'

function App() {
    const canvasHeight = 1080;
    const canvasWidth = 1920;
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const fileHandler = (e: ChangeEvent<HTMLInputElement>) => {
        const files = e.currentTarget.files;

        if (!files || files?.length === 0) return;

        const file = files[0];
        const reader = new FileReader();
        reader.addEventListener("load", () => {
            const res = reader.result as ArrayBuffer;

            const railway_map = RailwayMap.load(new Uint8Array(res));
            const renderInfo = railway_map.render(1000000000 - 5000, 1000000000 - 5000, 1080, 1920, 50);

            const canvas = canvasRef.current!;
            const ctx = canvas.getContext("2d")!;

            renderMap(ctx, renderInfo);
        });

        reader.readAsArrayBuffer(file);
    };

    return (<div>
        <div>
            <input type="file" onChange={fileHandler} />
        </div>
        <div>
            <canvas height={canvasHeight} width={canvasWidth} ref={canvasRef} />
        </div>
    </div>);
}

export default App;
