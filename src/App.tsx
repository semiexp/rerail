import { ChangeEvent, useState } from 'react';
import { RailwayMap } from '../rerail-internal/pkg/rerail_internal'
import { RailwayMapCanvas } from './RailwayMapCanvas'

type RerailAppState = {
    viewportHeight: number,
    viewportWidth: number,
    viewportTopX: number,
    viewportTopY: number,
    viewportZoom: number,
    railwayMap: RailwayMap | null,
};

function App() {
    const [appState, setAppState] = useState<RerailAppState>({
        viewportHeight: 1080,
        viewportWidth: 1920,
        viewportTopX: 1000000000,
        viewportTopY: 1000000000,
        viewportZoom: 50,
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

    return (<div>
        <div>
            <input type="file" onChange={fileHandler} />
        </div>
        <div>
            <RailwayMapCanvas
                height={appState.viewportHeight}
                width={appState.viewportWidth}
                topX={appState.viewportTopX}
                topY={appState.viewportTopY}
                zoom={appState.viewportZoom}
                setTopPos={(x, y) => setAppState({...appState, viewportTopX: x, viewportTopY: y})}
                railwayMap={appState.railwayMap}
            />
        </div>
    </div>);
}

export default App;
