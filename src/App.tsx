import { ChangeEvent, useEffect, useRef, useState } from "react";
import { RerailMap } from "./RerailMap";
import { RerailEditor, EditorMode } from "./RerailEditor";

import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import iconMove from "./assets/move.svg";
import iconRailway from "./assets/railway.svg";
import iconStation from "./assets/station.svg";
import iconBorders from "./assets/borders.svg";
import { ButtonGroup, IconButton } from "@mui/material";
import { FileOpen, Save } from "@mui/icons-material";
import { BorderSelecter } from "./BorderSelecter";

type RerailAppState = {
  viewportTopX: number;
  viewportTopY: number;
  viewportZoomLevel: number;
  railwayMap: RerailMap | null;
  editorMode: EditorMode;
  selectedBorderStyle: number;
};

function App() {
  const [appState, setAppState] = useState<RerailAppState>({
    viewportTopX: 1000000000,
    viewportTopY: 1000000000,
    viewportZoomLevel: 5,
    railwayMap: null,
    editorMode: "move",
    selectedBorderStyle: 0,
  });

  const fileElementRef = useRef<HTMLInputElement>(null);
  const anchorElementRef = useRef<HTMLAnchorElement>(null);
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
  const downloadMap = () => {
    const map = appState.railwayMap;
    if (map === null) {
      return;
    }
    const data = map.save();
    const blob = new Blob([data], { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);
    const a = anchorElementRef.current!;
    a.download = "map.rrl"; // TODO
    a.href = url;
    a.click();
    URL.revokeObjectURL(url);
  };
  const onKeyDown = (e: KeyboardEvent) => {
    // do not respond if the focus is on an input element
    if (document.activeElement instanceof HTMLInputElement) {
      return;
    }
    if (e.key === "m") {
      setAppState((appState) => ({
        ...appState,
        editorMode: "move",
      }));
    } else if (e.key === "r") {
      setAppState((appState) => ({
        ...appState,
        editorMode: "railway",
      }));
    } else if (e.key === "s") {
      setAppState((appState) => ({
        ...appState,
        editorMode: "station",
      }));
    } else if (e.key === "b") {
      setAppState((appState) => ({
        ...appState,
        editorMode: "borders",
      }));
    }
  };

  // add keyboard event handler to body component
  useEffect(() => {
    document.body.addEventListener("keydown", onKeyDown);
    return () => {
      document.body.removeEventListener("keydown", onKeyDown);
    };
  }, []);

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
      <div
        style={{
          width: "100%",
          backgroundColor: "#eeeeee",
          borderBottomWidth: "3px",
          borderBottomStyle: "solid",
          borderBottomColor: "#333333",
          display: "flex",
        }}
      >
        <input
          type="file"
          ref={fileElementRef}
          onChange={fileHandler}
          style={{ display: "none" }}
        />
        <a ref={anchorElementRef} style={{ display: "none" }} />
        <ButtonGroup>
          <IconButton
            size="small"
            onClick={() => fileElementRef.current!.click()}
          >
            <FileOpen />
          </IconButton>
          <IconButton
            size="small"
            onClick={downloadMap}
            disabled={appState.railwayMap === null}
          >
            <Save />
          </IconButton>
        </ButtonGroup>
        <div
          style={{
            width: "1px",
            backgroundColor: "#888888",
            marginLeft: "5px",
            marginRight: "5px",
          }}
        />
        <ToggleButtonGroup value={appState.editorMode}>
          <ToggleButton
            value="move"
            size="small"
            sx={{ padding: 0.2 }}
            onClick={() => setAppState({ ...appState, editorMode: "move" })}
            disableRipple
          >
            <img src={iconMove} height={24} />
          </ToggleButton>
          <ToggleButton
            value="railway"
            size="small"
            sx={{ padding: 0.2 }}
            onClick={() => setAppState({ ...appState, editorMode: "railway" })}
            disableRipple
          >
            <img src={iconRailway} height={24} />
          </ToggleButton>
          <ToggleButton
            value="station"
            size="medium"
            sx={{ padding: 0.2 }}
            onClick={() => setAppState({ ...appState, editorMode: "station" })}
            disableRipple
          >
            <img src={iconStation} height={24} />
          </ToggleButton>
          <ToggleButton
            value="borders"
            size="medium"
            sx={{ padding: 0.2 }}
            onClick={() => setAppState({ ...appState, editorMode: "borders" })}
            disableRipple
          >
            <img src={iconBorders} height={24} />
          </ToggleButton>
          <BorderSelecter
            value={appState.selectedBorderStyle}
            onChange={(v) =>
              setAppState({ ...appState, selectedBorderStyle: v })
            }
          />
        </ToggleButtonGroup>
      </div>
      <div style={{ flex: 1, overflow: "hidden" }}>
        <RerailEditor
          topX={appState.viewportTopX}
          topY={appState.viewportTopY}
          zoomLevel={appState.viewportZoomLevel}
          editorMode={appState.editorMode}
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
          newBorderStyle={appState.selectedBorderStyle}
        />
      </div>
    </div>
  );
}

export default App;
