import { ChangeEvent, useEffect, useRef, useState } from 'react';
import { RailwayMap } from '../rerail-internal/pkg/rerail_internal'
import { RailwayMapCanvas } from './RailwayMapCanvas'

type RerailAppState = {
    viewportHeight: number,
    viewportWidth: number,
    viewportTopX: number,
    viewportTopY: number,
    viewportZoomLevel: number,
    railwayMap: RailwayMap | null,
};

function App() {
    const [appState, setAppState] = useState<RerailAppState>({
        viewportHeight: 100,
        viewportWidth: 100,
        viewportTopX: 1000000000,
        viewportTopY: 1000000000,
        viewportZoomLevel: 5,
        railwayMap: null,
    });

    const fileHandler = (e: ChangeEvent<HTMLInputElement>) => {
        const files = e.currentTarget.files;

        if (!files || files?.length === 0) return;

        const file = files[0];
        const reader = new FileReader();
        reader.addEventListener("load", () => {
            const res = reader.result as ArrayBuffer;

            const railwayMap = RailwayMap.load(new Uint8Array(res));
            setAppState({...appState, railwayMap})
        });

        reader.readAsArrayBuffer(file);
    };

    const canvasContainerRef = useRef<HTMLDivElement>(null);

    const setSize = () => {
        const container = canvasContainerRef.current!;
        const height = container.clientHeight;
        const width = container.clientWidth;

        setAppState(st => {return {...st, viewportHeight: height, viewportWidth: width}});
    };

    useEffect(() => {
        window.addEventListener("resize", () => {
            setSize();
        });
        setSize();
    }, []);

    return (<div style={{display: "flex", flexDirection: "column", height: "100%", width: "100%", margin: 0, padding: 0, position: "absolute"}}>
        <div style={{width: "100%"}}>
            <input type="file" onChange={fileHandler} />
        </div>
        <div style={{flex: 1, height: "100%", width: "100%", minHeight: "100px", overflow: "none"}} ref={canvasContainerRef}>
            <RailwayMapCanvas
                height={appState.viewportHeight}
                width={appState.viewportWidth}
                topX={appState.viewportTopX}
                topY={appState.viewportTopY}
                zoomLevel={appState.viewportZoomLevel}
                setViewport={(x, y, zoomLevel) => setAppState({...appState, viewportTopX: x, viewportTopY: y, viewportZoomLevel: zoomLevel})}
                railwayMap={appState.railwayMap}
            />
        </div>
    </div>);
}

export default App;
