import { useState, useRef, useEffect } from "react";
import {
  RerailMap,
  ViewportSpec,
  ViewportRailwayList,
  NearestSegment,
} from "./RerailMap";
import { renderMap } from "./renderer";
import { RailwayListViewer } from "./RailwayListViewer";
import { StationDialog, StationDialogRefType } from "./Dialogs";

type EditorMode = "move" | "railway" | "station";

type RerailEditorProps = {
  topX: number;
  topY: number;
  zoomLevel: number;
  editorMode: EditorMode;
  setViewport: (x: number, y: number, zoomLevel: number) => void;
  setRailwayMap: (map: RerailMap) => void;
  railwayMap: RerailMap | null;
};

type EditorPhase = "none" | "viewport-moving" | "point-moving";

type RerailEditorStateType = {
  canvasHeight: number;
  canvasWidth: number;
  railwayList: ViewportRailwayList | null;
  selectedRailId: number | null;
  editorPhase: EditorPhase;

  // viewport-moving
  mouseXOnMouseDown?: number;
  mouseYOnMouseDown?: number;
  topXOnMouseDown?: number;
  topYOnMouseDown?: number;

  // point-moving
  skipNearestSegment?: NearestSegment;
  mouse?: { x: number; y: number };
};

const initialRerailEditorState: RerailEditorStateType = {
  canvasHeight: 100,
  canvasWidth: 100,
  railwayList: null,
  selectedRailId: null,
  editorPhase: "none",
};

const zoomLevels = [
  1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000,
];

export const RerailEditor = (props: RerailEditorProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const entireContainerRef = useRef<HTMLDivElement>(null);
  const canvasContainerRef = useRef<HTMLDivElement>(null);
  const stationDialogRef = useRef<StationDialogRefType>(null);

  const [state, setState] = useState<RerailEditorStateType>(
    initialRerailEditorState,
  );

  const isShiftKeyPressed = useRef(false);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      isShiftKeyPressed.current = e.shiftKey;
    };
    window.addEventListener("keydown", handler);
    window.addEventListener("keyup", handler);
    return () => {
      window.removeEventListener("keydown", handler);
      window.removeEventListener("keyup", handler);
    };
  }, []);

  useEffect(() => {
    if (!props.railwayMap) return;
    const railwayMap = props.railwayMap;
    const viewport: ViewportSpec = {
      leftX: props.topX,
      topY: props.topY,
      height: state.canvasHeight,
      width: state.canvasWidth,
      zoom: zoomLevels[props.zoomLevel],
    };

    const renderInfo = railwayMap.render(viewport, {
      selectedRailId:
        state.selectedRailId !== null ? state.selectedRailId : undefined,
      skipNearestSegment: state.skipNearestSegment,
      mouse: state.mouse,
    });

    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;

    renderMap(ctx, state.canvasWidth, state.canvasHeight, renderInfo);

    const railwayList = railwayMap.railwaysInViewport(viewport);
    setState((state) => {
      return { ...state, railwayList };
    });
  }, [
    props,
    state.canvasHeight,
    state.canvasWidth,
    state.selectedRailId,
    state.skipNearestSegment,
    state.mouse,
  ]);

  const canvasMouseDownHandler = async (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    if (props.railwayMap === null) {
      return;
    }
    if (state.editorPhase !== "none") {
      return;
    }

    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    const editorMode = props.editorMode;
    const transitionToViewportMoving =
      editorMode === "move" || isShiftKeyPressed.current;
    const transitionToPointMoving =
      !transitionToViewportMoving &&
      editorMode === "railway" &&
      state.selectedRailId !== null;
    const maybeOpenStationEditor =
      editorMode === "station" &&
      !transitionToViewportMoving &&
      !transitionToPointMoving;

    if (transitionToViewportMoving) {
      setState({
        ...state,
        editorPhase: "viewport-moving",
        mouseXOnMouseDown: x,
        mouseYOnMouseDown: y,
        topXOnMouseDown: props.topX,
        topYOnMouseDown: props.topY,
      });
    }
    if (transitionToPointMoving) {
      const viewport: ViewportSpec = {
        leftX: props.topX,
        topY: props.topY,
        height: state.canvasHeight,
        width: state.canvasWidth,
        zoom: zoomLevels[props.zoomLevel],
      };
      const nearest = props.railwayMap?.findNearestSegment(
        viewport,
        state.selectedRailId!,
        x,
        y,
        10,
      );
      if (nearest) {
        setState({
          ...state,
          editorPhase: "point-moving",
          skipNearestSegment: nearest,
          mouse: { x, y },
        });
      }
    }
    if (maybeOpenStationEditor) {
      const viewport: ViewportSpec = {
        leftX: props.topX,
        topY: props.topY,
        height: state.canvasHeight,
        width: state.canvasWidth,
        zoom: zoomLevels[props.zoomLevel],
      };
      const nearest = props.railwayMap?.findNearestSegment(
        viewport,
        state.selectedRailId!,
        x,
        y,
        10,
      );
      if (nearest && !nearest.betweenPoints) {
        const index = nearest.index;
        const stationInfo = props.railwayMap!.getStationInfo(
          state.selectedRailId!,
          index,
        );
        const stationValue = await stationDialogRef.current!.open(
          stationInfo || { name: "", level: 0 },
        );
        if (stationValue && stationValue.name !== "") {
          props.setRailwayMap(
            props.railwayMap!.setStationInfo(
              state.selectedRailId!,
              index,
              stationValue,
            ),
          );
        }
      }
    }
  };

  const canvasMouseMoveHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    if (state.editorPhase === "viewport-moving") {
      const zoom = zoomLevels[props.zoomLevel];
      const newTopX =
        state.topXOnMouseDown! + (state.mouseXOnMouseDown! - x) * zoom;
      const newTopY =
        state.topYOnMouseDown! + (state.mouseYOnMouseDown! - y) * zoom;
      props.setViewport(newTopX, newTopY, props.zoomLevel);
    } else if (state.editorPhase === "point-moving") {
      setState({
        ...state,
        mouse: { x, y },
      });
    }
  };

  const canvasMouseUpHandler = () => {
    if (state.editorPhase === "viewport-moving") {
      setState({
        ...state,
        editorPhase: "none",
        mouseXOnMouseDown: undefined,
        mouseYOnMouseDown: undefined,
        topXOnMouseDown: undefined,
        topYOnMouseDown: undefined,
      });
    } else if (state.editorPhase === "point-moving") {
      const x = state.mouse!.x * zoomLevels[props.zoomLevel] + props.topX;
      const y = state.mouse!.y * zoomLevels[props.zoomLevel] + props.topY;

      const map = props.railwayMap!;
      if (state.skipNearestSegment!.betweenPoints) {
        props.setRailwayMap(
          map.insertRailwayPoint(
            state.selectedRailId!,
            state.skipNearestSegment!.index + 1,
            x,
            y,
          ),
        );
      } else {
        props.setRailwayMap(
          map.moveRailwayPoint(
            state.selectedRailId!,
            state.skipNearestSegment!.index,
            x,
            y,
          ),
        );
      }
      setState({
        ...state,
        editorPhase: "none",
        skipNearestSegment: undefined,
        mouse: undefined,
      });
    }
  };

  const canvasWheelHandler = (e: React.WheelEvent) => {
    if (state.editorPhase !== "none") {
      return;
    }

    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    const zoomLevel = props.zoomLevel;
    const newZoomLevel = zoomLevel + (e.deltaY < 0 ? -1 : 1);

    if (!(0 <= newZoomLevel && newZoomLevel < zoomLevels.length)) return;

    const newTopX =
      props.topX + x * (zoomLevels[zoomLevel] - zoomLevels[newZoomLevel]);
    const newTopY =
      props.topY + y * (zoomLevels[zoomLevel] - zoomLevels[newZoomLevel]);
    props.setViewport(newTopX, newTopY, newZoomLevel);
  };

  const [sliderX, setSliderX] = useState<number | null>(null);
  const [sidebarWidth, setSidebarWidth] = useState(100);
  const sliderMouseDownHandler = (
    e: React.MouseEvent<HTMLDivElement, MouseEvent>,
  ) => {
    const x = e.clientX - (e.target as HTMLDivElement).offsetLeft;
    setSliderX(x);
  };
  const sliderMouseMoveHandler = (
    e: React.MouseEvent<HTMLDivElement, MouseEvent>,
  ) => {
    if (sliderX === null) return;
    const x = e.clientX - entireContainerRef.current!.offsetLeft;
    const newSidebarWidth = Math.max(100, x - sliderX);
    setSidebarWidth(newSidebarWidth);
  };
  const sliderMouseUpHandler = () => setSliderX(null);
  const sliderMouseOutHandler = (
    e: React.MouseEvent<HTMLDivElement, MouseEvent>,
  ) => {
    if (e.target === entireContainerRef.current) {
      setSliderX(null);
    }
  };

  const setCanvasSize = () => {
    const container = canvasContainerRef.current!;
    const height = container.clientHeight;
    const width = container.clientWidth;

    setState((state) => {
      return { ...state, canvasHeight: height, canvasWidth: width };
    });
  };

  useEffect(setCanvasSize, [sidebarWidth]);
  useEffect(() => {
    window.addEventListener("resize", () => {
      setCanvasSize();
    });
    setCanvasSize();
  }, []);

  const onSelectRailway = (id: number) => {
    setState((state) => {
      return { ...state, selectedRailId: id };
    });
  };

  let cursor = "none";
  if (state.editorPhase === "viewport-moving") {
    cursor = "move";
  } else if (state.editorPhase === "point-moving") {
    cursor = "pointer";
  }

  return (
    <div
      ref={entireContainerRef}
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "row",
        cursor: sliderX !== null ? "col-resize" : "auto",
      }}
      onMouseMove={sliderMouseMoveHandler}
      onMouseUp={sliderMouseUpHandler}
      onMouseOut={sliderMouseOutHandler}
    >
      <div
        style={{
          width: sidebarWidth,
          minWidth: sidebarWidth,
          height: "100%",
          backgroundColor: "#eeeeee",
        }}
      >
        {state.railwayList && (
          <RailwayListViewer
            railwayList={state.railwayList}
            selectedId={state.selectedRailId}
            onSelect={onSelectRailway}
          />
        )}
      </div>
      <div
        style={{
          width: 5,
          minWidth: 5,
          height: "100%",
          backgroundColor: "#666666",
          cursor: "col-resize",
        }}
        onMouseDown={sliderMouseDownHandler}
      ></div>
      <div
        ref={canvasContainerRef}
        style={{ flex: 1, height: "100%", overflow: "hidden" }}
      >
        <canvas
          height={state.canvasHeight}
          width={state.canvasWidth}
          ref={canvasRef}
          onMouseDown={canvasMouseDownHandler}
          onMouseMove={canvasMouseMoveHandler}
          onMouseUp={canvasMouseUpHandler}
          onMouseOut={canvasMouseUpHandler}
          onWheel={canvasWheelHandler}
          style={{
            verticalAlign: "top",
            ...(state.editorPhase !== "none" ? { cursor } : {}),
          }}
        />
      </div>
      <StationDialog ref={stationDialogRef}></StationDialog>
    </div>
  );
};
