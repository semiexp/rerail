import { useState, useRef, useEffect } from "react";
import {
  RerailMap,
  Viewport,
  ViewportRailwayList,
  NearestSegment,
} from "./RerailMap";
import { renderMap } from "./renderer";
import { RailwayListViewer } from "./RailwayListViewer";

type RerailEditorProps = {
  topX: number;
  topY: number;
  zoomLevel: number;
  setViewport: (x: number, y: number, zoomLevel: number) => void;
  setRailwayMap: (map: RerailMap) => void;
  railwayMap: RerailMap | null;
};

type EditorMode = "none" | "viewport-moving" | "point-moving";

type RerailEditorStateType = {
  canvasHeight: number;
  canvasWidth: number;
  railwayList: ViewportRailwayList | null;
  selectedRailId: number | null;
  editorMode: EditorMode;

  // viewport-moving
  mouseXOnMouseDown?: number;
  mouseYOnMouseDown?: number;
  topXOnMouseDown?: number;
  topYOnMouseDown?: number;

  // point-moving
  skipNearestSegment?: NearestSegment;
  mouseX?: number;
  mouseY?: number;
};

const initialRerailEditorState: RerailEditorStateType = {
  canvasHeight: 100,
  canvasWidth: 100,
  railwayList: null,
  selectedRailId: null,
  editorMode: "none",
};

const zoomLevels = [
  1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000,
];

export const RerailEditor = (props: RerailEditorProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const entireContainerRef = useRef<HTMLDivElement>(null);
  const canvasContainerRef = useRef<HTMLDivElement>(null);

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
    const viewport: Viewport = {
      leftX: props.topX,
      topY: props.topY,
      height: state.canvasHeight,
      width: state.canvasWidth,
      zoom: zoomLevels[props.zoomLevel],
    };

    const renderInfo = railwayMap.render(
      viewport,
      state.selectedRailId === null ? undefined : state.selectedRailId,
      state.skipNearestSegment && state.skipNearestSegment.clone(),
      state.mouseX,
      state.mouseY,
    );

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
    state.mouseX,
    state.mouseY,
  ]);

  const canvasMouseDownHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    if (props.railwayMap === null) {
      return;
    }
    if (state.editorMode !== "none") {
      return;
    }

    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    if (state.selectedRailId !== null && !isShiftKeyPressed.current) {
      const viewport: Viewport = {
        leftX: props.topX,
        topY: props.topY,
        height: state.canvasHeight,
        width: state.canvasWidth,
        zoom: zoomLevels[props.zoomLevel],
      };
      const nearest = props.railwayMap?.findNearestSegment(
        viewport,
        state.selectedRailId,
        x,
        y,
        10,
      );
      if (nearest) {
        setState({
          ...state,
          editorMode: "point-moving",
          skipNearestSegment: nearest,
          mouseX: x,
          mouseY: y,
        });
      }
      return;
    } else {
      setState({
        ...state,
        editorMode: "viewport-moving",
        mouseXOnMouseDown: x,
        mouseYOnMouseDown: y,
        topXOnMouseDown: props.topX,
        topYOnMouseDown: props.topY,
      });
    }
  };

  const canvasMouseMoveHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    if (state.editorMode === "viewport-moving") {
      const zoom = zoomLevels[props.zoomLevel];
      const newTopX =
        state.topXOnMouseDown! + (state.mouseXOnMouseDown! - x) * zoom;
      const newTopY =
        state.topYOnMouseDown! + (state.mouseYOnMouseDown! - y) * zoom;
      props.setViewport(newTopX, newTopY, props.zoomLevel);
    } else if (state.editorMode === "point-moving") {
      setState({
        ...state,
        mouseX: x,
        mouseY: y,
      });
    }
  };

  const canvasMouseUpHandler = () => {
    if (state.editorMode === "viewport-moving") {
      setState({
        ...state,
        editorMode: "none",
        mouseXOnMouseDown: undefined,
        mouseYOnMouseDown: undefined,
        topXOnMouseDown: undefined,
        topYOnMouseDown: undefined,
      });
    } else if (state.editorMode === "point-moving") {
      const x = state.mouseX! * zoomLevels[props.zoomLevel] + props.topX;
      const y = state.mouseY! * zoomLevels[props.zoomLevel] + props.topY;

      const map = props.railwayMap!;
      if (state.skipNearestSegment!.idx1) {
        props.setRailwayMap(
          map.insertRailwayPoint(
            state.selectedRailId!,
            state.skipNearestSegment!.idx0 + 1,
            x,
            y,
          ),
        );
      } else {
        props.setRailwayMap(
          map.moveRailwayPoint(
            state.selectedRailId!,
            state.skipNearestSegment!.idx0,
            x,
            y,
          ),
        );
      }
      setState({
        ...state,
        editorMode: "none",
        skipNearestSegment: undefined,
        mouseX: undefined,
        mouseY: undefined,
      });
    }
  };

  const canvasWheelHandler = (e: React.WheelEvent) => {
    if (state.editorMode !== "none") {
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
  if (state.editorMode === "viewport-moving") {
    cursor = "move";
  } else if (state.editorMode === "point-moving") {
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
            ...(state.editorMode !== "none" ? { cursor } : {}),
          }}
        />
      </div>
    </div>
  );
};
