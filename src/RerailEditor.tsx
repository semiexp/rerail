import { useState, useRef, useEffect } from "react";
import {
  RerailMap,
  ViewportRailwayList,
} from "../rerail-internal/pkg/rerail_internal";
import { renderMap } from "./renderer";
import { RailwayListViewer } from "./RailwayListViewer";

type RerailEditorProps = {
  topX: number;
  topY: number;
  zoomLevel: number;
  setViewport: (x: number, y: number, zoomLevel: number) => void;
  railwayMap: RerailMap | null;
};

type RerailEditorStateType = {
  canvasHeight: number;
  canvasWidth: number;
  railwayList: ViewportRailwayList | null;
  selectedRailId: number | null;
};

const initialRerailEditorState: RerailEditorStateType = {
  canvasHeight: 100,
  canvasWidth: 100,
  railwayList: null,
  selectedRailId: null,
};

type ClickInfo = {
  mouseX: number;
  mouseY: number;
  topX: number;
  topY: number;
};

const zoomLevels = [
  1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000,
];

export const RerailEditor = (props: RerailEditorProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const entireContainerRef = useRef<HTMLDivElement>(null);
  const canvasContainerRef = useRef<HTMLDivElement>(null);

  const [clickPos, setClickPos] = useState<ClickInfo | null>(null);
  const [state, setState] = useState<RerailEditorStateType>(
    initialRerailEditorState,
  );

  useEffect(() => {
    if (!props.railwayMap) return;
    const railwayMap = props.railwayMap;
    const renderInfo = railwayMap.render(
      props.topX,
      props.topY,
      state.canvasHeight,
      state.canvasWidth,
      zoomLevels[props.zoomLevel],
      state.selectedRailId === null ? undefined : state.selectedRailId,
    );

    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;

    renderMap(ctx, state.canvasWidth, state.canvasHeight, renderInfo);

    const railwayList = railwayMap.railways_in_viewport(
      props.topX,
      props.topY,
      state.canvasHeight,
      state.canvasWidth,
      zoomLevels[props.zoomLevel],
    );
    setState((state) => {
      return { ...state, railwayList };
    });
  }, [props, state.canvasHeight, state.canvasWidth, state.selectedRailId]);

  const canvasMouseDownHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    setClickPos({ mouseX: x, mouseY: y, topX: props.topX, topY: props.topY });
  };

  const canvasMouseMoveHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    if (clickPos === null) return;

    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    const zoom = zoomLevels[props.zoomLevel];
    const newTopX = clickPos.topX + (clickPos.mouseX - x) * zoom;
    const newTopY = clickPos.topY + (clickPos.mouseY - y) * zoom;
    props.setViewport(newTopX, newTopY, props.zoomLevel);
  };

  const canvasMouseUpHandler = () => setClickPos(null);

  const canvasWheelHandler = (e: React.WheelEvent) => {
    if (clickPos !== null) return;

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
            ...(clickPos !== null ? { cursor: "move" } : {}),
          }}
        />
      </div>
    </div>
  );
};
