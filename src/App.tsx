import { ChangeEvent, useRef, useState } from "react";
import { RerailMap } from "./RerailMap";
import { RerailEditor } from "./RerailEditor";

type RerailAppState = {
  viewportTopX: number;
  viewportTopY: number;
  viewportZoomLevel: number;
  railwayMap: RerailMap | null;
};

function App() {
  const [appState, setAppState] = useState<RerailAppState>({
    viewportTopX: 1000000000,
    viewportTopY: 1000000000,
    viewportZoomLevel: 5,
    railwayMap: null,
  });

  const fileElementRef = useRef<HTMLInputElement>(null);
  const fileHandler = (e: ChangeEvent<HTMLInputElement>) => {
    const files = e.currentTarget.files;

    if (!files || files?.length === 0) return;

    const file = files[0];
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      const res = reader.result as ArrayBuffer;

      const railwayMap = RerailMap.load(new Uint8Array(res));
      setAppState({ ...appState, railwayMap });
    });

    reader.readAsArrayBuffer(file);
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        width: "100%",
        margin: 0,
        padding: 0,
      }}
    >
      <div style={{ width: "100%" }}>
        <input
          type="button"
          value="Open"
          onClick={() => fileElementRef.current!.click()}
        />
        <input
          type="file"
          ref={fileElementRef}
          onChange={fileHandler}
          style={{ display: "none" }}
        />
      </div>
      <div style={{ flex: 1, overflow: "hidden" }}>
        <RerailEditor
          topX={appState.viewportTopX}
          topY={appState.viewportTopY}
          zoomLevel={appState.viewportZoomLevel}
          setViewport={(x, y, zoomLevel) =>
            setAppState({
              ...appState,
              viewportTopX: x,
              viewportTopY: y,
              viewportZoomLevel: zoomLevel,
            })
          }
          setRailwayMap={(map) => setAppState({ ...appState, railwayMap: map })}
          railwayMap={appState.railwayMap}
        />
      </div>
    </div>
  );
}

export default App;
