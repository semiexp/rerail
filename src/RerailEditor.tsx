import { useState, useRef, useEffect } from "react";
import {
  RerailMap,
  ViewportSpec,
  ViewportRailwayList,
  IndexOnRailway,
  BorderPointOrSegment,
} from "./RerailMap";
import { renderMap } from "./renderer";
import { RailwayListViewer } from "./RailwayListViewer";
import {
  RailwayDialog,
  RailwayDialogRefType,
  StationDialog,
  StationDialogRefType,
  StationListDialog,
  StationListDialogRefType,
} from "./Dialogs";

export type EditorMode =
  | "move"
  | "newRailway"
  | "railway"
  | "station"
  | "borders";

type RerailEditorProps = {
  topX: number;
  topY: number;
  zoomLevel: number;
  editorMode: EditorMode;
  setViewport: (x: number, y: number, zoomLevel: number) => void;
  setRailwayMap: (map: RerailMap) => void;
  railwayMap: RerailMap | null;
  newBorderStyle: number;
};

type EditorPhase =
  | "none"
  | "viewport-moving"
  | "point-moving"
  | "new-rail"
  | "station-linking"
  | "border-moving"
  | "border-adding";

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

  // point-moving & station-linking
  selectedIndex?: IndexOnRailway;

  // station-linking
  moved?: boolean;

  // border-moving
  selectedBorderIndex?: BorderPointOrSegment;

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

const distanceThreshold: number = 10;

export const RerailEditor = (props: RerailEditorProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const entireContainerRef = useRef<HTMLDivElement>(null);
  const canvasContainerRef = useRef<HTMLDivElement>(null);
  const stationDialogRef = useRef<StationDialogRefType>(null);
  const railwayDialogRef = useRef<RailwayDialogRefType>(null);
  const stationListDialogRef = useRef<StationListDialogRefType>(null);

  const [state, setState] = useState<RerailEditorStateType>(
    initialRerailEditorState,
  );

  const viewport = {
    leftX: props.topX,
    topY: props.topY,
    height: state.canvasHeight,
    width: state.canvasWidth,
    zoom: zoomLevels[props.zoomLevel],
  };

  useEffect(() => {
    if (!props.railwayMap) return;
    const railwayMap = props.railwayMap;

    let temporaryMovingPoint = undefined;
    if (
      state.editorPhase === "point-moving" ||
      (state.editorPhase === "station-linking" && state.moved)
    ) {
      temporaryMovingPoint = {
        index: state.selectedIndex!,
        pointAfterMove: state.mouse!,
      };
    } else if (state.editorPhase === "new-rail") {
      // index should be IndexOnRailway
      temporaryMovingPoint = {
        index: {
          index: railwayMap.getNumberOfPointsOnRailway(state.selectedRailId!),
          inserting: true,
        },
        pointAfterMove: state.mouse!,
      };
    }
    let temporaryMovingBorderPoint = undefined;
    if (state.editorPhase === "border-moving") {
      temporaryMovingBorderPoint = {
        pointOrSegment: state.selectedBorderIndex!,
        pointAfterMove: state.mouse!,
      };
    }
    let extraBorderSegment = undefined;
    if (state.editorPhase === "border-adding") {
      const idx = state.selectedBorderIndex!;
      if ("point" in idx) {
        extraBorderSegment = {
          point: idx.point,
          newPoint: state.mouse!,
          level: props.newBorderStyle,
        };
      }
    }
    const renderInfo = railwayMap.render(viewport, {
      selectedRailId:
        state.selectedRailId !== null && props.editorMode !== "borders"
          ? state.selectedRailId
          : undefined,
      temporaryMovingPoint,
      markerOnBorderPoints: props.editorMode === "borders",
      temporaryMovingBorderPoint,
      extraBorderSegment,
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
    state.selectedIndex,
    state.selectedBorderIndex,
    state.mouse,
  ]);

  const canvasMouseDownHandler = async (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    if (props.railwayMap === null) {
      return;
    }
    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    if (state.editorPhase === "new-rail") {
      const numPoints = props.railwayMap!.getNumberOfPointsOnRailway(
        state.selectedRailId!,
      );

      if (e.button === 2) {
        // if there is only one point, remove the railway
        if (numPoints === 1) {
          // TODO: implement removeRailway
          props.setRailwayMap(
            props.railwayMap!.removeRailway(state.selectedRailId!),
          );
          setState({
            ...state,
            selectedRailId: null,
            editorPhase: "none",
          });
        } else {
          setState({
            ...state,
            editorPhase: "none",
          });
        }
        return;
      }

      const logicalX = x * zoomLevels[props.zoomLevel] + props.topX;
      const logicalY = y * zoomLevels[props.zoomLevel] + props.topY;
      // add (logX, logY) to selected rail
      props.setRailwayMap(
        props.railwayMap!.insertRailwayPoint(
          state.selectedRailId!,
          numPoints,
          logicalX,
          logicalY,
        ),
      );
      return;
    }
    if (state.editorPhase !== "none") {
      return;
    }

    const editorMode = props.editorMode;
    if (editorMode === "move" || e.shiftKey) {
      setState({
        ...state,
        editorPhase: "viewport-moving",
        mouseXOnMouseDown: x,
        mouseYOnMouseDown: y,
        topXOnMouseDown: props.topX,
        topYOnMouseDown: props.topY,
      });
    } else if (editorMode === "newRailway") {
      // get railway info using dialog, then add a new railway with point (x, y) and the info
      const railwayInfo = await railwayDialogRef.current!.open({
        name: "",
        color: 0,
        level: 0,
      });
      if (railwayInfo === undefined) {
        return;
      }
      const logicalX = x * zoomLevels[props.zoomLevel] + props.topX;
      const logicalY = y * zoomLevels[props.zoomLevel] + props.topY;
      const newMapAndIndex = props.railwayMap!.newRailwayFromInfo(
        railwayInfo,
        logicalX,
        logicalY,
      );
      const newMap = newMapAndIndex.getMap();
      const newIndex = newMapAndIndex.getRailwayIndex();

      props.setRailwayMap(newMap);
      setState({
        ...state,
        editorPhase: "new-rail",
        selectedRailId: newIndex,
      });
    } else if (editorMode === "railway") {
      if (state.selectedRailId === null) {
        return;
      }
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
        distanceThreshold,
      );
      // if ctrl-key is pressed, extend the railway
      if (e.ctrlKey) {
        if (nearest && !nearest.inserting) {
          const numPoints = props.railwayMap!.getNumberOfPointsOnRailway(
            state.selectedRailId!,
          );
          if (nearest.index === 0) {
            setState({
              ...state,
              editorPhase: "point-moving",
              selectedIndex: { index: 0, inserting: true },
              mouse: { x, y },
            });
          } else if (nearest.index === numPoints - 1) {
            setState({
              ...state,
              editorPhase: "point-moving",
              selectedIndex: { index: numPoints, inserting: true },
              mouse: { x, y },
            });
          }
        }
      } else if (e.button === 2) {
        if (nearest && !nearest.inserting) {
          props.setRailwayMap(
            props.railwayMap!.removeRailwayPoint(
              state.selectedRailId!,
              nearest.index,
            ),
          );
        }
      } else {
        if (nearest) {
          setState({
            ...state,
            editorPhase: "point-moving",
            selectedIndex: nearest,
            mouse: { x, y },
          });
        }
      }
    } else if (editorMode === "station") {
      if (state.selectedRailId === null) {
        return;
      }
      const nearest = props.railwayMap?.findNearestSegment(
        viewport,
        state.selectedRailId!,
        x,
        y,
        distanceThreshold,
      );
      if (nearest && !nearest.inserting) {
        const index = nearest.index;

        if (e.button === 2) {
          const stationInfo = props.railwayMap!.getStationInfo(
            state.selectedRailId!,
            index,
          );
          if (stationInfo !== undefined) {
            props.setRailwayMap(
              props.railwayMap!.detachStationOnRailway(
                state.selectedRailId!,
                nearest.index,
              ),
            );
          }
        } else {
          setState({
            ...state,
            editorPhase: "station-linking",
            selectedIndex: nearest,
            mouse: { x, y },
            moved: false,
          });
        }
      }
    } else if (editorMode === "borders") {
      const nearest = props.railwayMap?.findNearestBorder(
        viewport,
        x,
        y,
        distanceThreshold,
      );
      if (nearest) {
        if (e.ctrlKey) {
          if ("point" in nearest) {
            setState({
              ...state,
              editorPhase: "border-adding",
              selectedBorderIndex: nearest,
              mouse: { x, y },
            });
          }
        } else {
          if (e.button === 2) {
            if ("point" in nearest) {
              props.setRailwayMap(
                props.railwayMap!.removeBorderPoint(nearest.point),
              );
            } else {
              props.setRailwayMap(
                props.railwayMap!.removeBorderEdge(
                  nearest.segment[0],
                  nearest.segment[1],
                ),
              );
            }
          } else {
            setState({
              ...state,
              editorPhase: "border-moving",
              selectedBorderIndex: nearest,
              mouse: { x, y },
            });
          }
        }
      }
    }
  };

  const canvasMouseMoveHandler = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ) => {
    const x = e.clientX - (e.target as HTMLCanvasElement).offsetLeft;
    const y = e.clientY - (e.target as HTMLCanvasElement).offsetTop;

    setState((prevState) => ({
      ...prevState,
      mouse: { x, y },
    }));

    if (state.editorPhase === "viewport-moving") {
      const zoom = zoomLevels[props.zoomLevel];
      const newTopX =
        state.topXOnMouseDown! + (state.mouseXOnMouseDown! - x) * zoom;
      const newTopY =
        state.topYOnMouseDown! + (state.mouseYOnMouseDown! - y) * zoom;
      props.setViewport(newTopX, newTopY, props.zoomLevel);
    } else if (state.editorPhase === "station-linking") {
      setState((prevState) => ({
        ...prevState,
        moved: true,
      }));
    }
  };

  const canvasMouseUpHandler = async () => {
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
      if (state.selectedIndex!.inserting) {
        props.setRailwayMap(
          map.insertRailwayPoint(
            state.selectedRailId!,
            state.selectedIndex!.index,
            x,
            y,
          ),
        );
      } else {
        props.setRailwayMap(
          map.moveRailwayPoint(
            state.selectedRailId!,
            state.selectedIndex!.index,
            x,
            y,
          ),
        );
      }
      setState({
        ...state,
        editorPhase: "none",
        selectedIndex: undefined,
      });
    } else if (state.editorPhase === "station-linking") {
      if (state.moved) {
        // link station
        props.setRailwayMap(
          props.railwayMap!.linkToStation(
            state.selectedRailId!,
            state.selectedIndex!.index,
            viewport,
            state.mouse!,
          ),
        );
        setState({
          ...state,
          editorPhase: "none",
          selectedIndex: undefined,
          moved: undefined,
        });
      } else {
        const index = state.selectedIndex!.index;
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
        setState({
          ...state,
          editorPhase: "none",
          selectedIndex: undefined,
          moved: undefined,
        });
      }
    } else if (state.editorPhase === "border-moving") {
      const selected = state.selectedBorderIndex!;
      const x = state.mouse!.x * zoomLevels[props.zoomLevel] + props.topX;
      const y = state.mouse!.y * zoomLevels[props.zoomLevel] + props.topY;

      if ("point" in selected) {
        props.setRailwayMap(
          props.railwayMap!.moveBorderPoint(selected.point, x, y),
        );
      } else {
        props.setRailwayMap(
          props.railwayMap!.insertBorderPointBetweenSegment(
            selected.segment[0],
            selected.segment[1],
            x,
            y,
          ),
        );
      }
      setState({
        ...state,
        editorPhase: "none",
        selectedBorderIndex: undefined,
      });
    } else if (state.editorPhase === "border-adding") {
      const map = props.railwayMap!;
      const x = state.mouse!.x * zoomLevels[props.zoomLevel] + props.topX;
      const y = state.mouse!.y * zoomLevels[props.zoomLevel] + props.topY;

      const selected = state.selectedBorderIndex!;
      if ("point" in selected) {
        const target = map.findNearestBorder(
          viewport,
          state.mouse!.x,
          state.mouse!.y,
          distanceThreshold,
        );
        if (target !== undefined && "point" in target) {
          props.setRailwayMap(
            map.connectExistingBorderPoints(
              selected.point,
              target.point,
              props.newBorderStyle,
            ),
          );
        } else {
          props.setRailwayMap(
            map.connectToNewBorderPoint(
              selected.point,
              x,
              y,
              props.newBorderStyle,
            ),
          );
        }
      }

      setState({
        ...state,
        editorPhase: "none",
        selectedBorderIndex: undefined,
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
    window.addEventListener("resize", setCanvasSize);
    setCanvasSize();

    return () => {
      window.removeEventListener("resize", setCanvasSize);
    };
  }, []);

  const onSelectRailway = (id: number) => {
    setState((state) => {
      return { ...state, selectedRailId: id };
    });
  };

  const onOpenRailwayConfig = async (id: number) => {
    const railwayMap = props.railwayMap;
    if (railwayMap === null) {
      return;
    }
    const railwayInfo = railwayMap.getRailwayInfo(id);
    const newRailwayInfo = await railwayDialogRef.current!.open(railwayInfo);
    if (newRailwayInfo === undefined) {
      return;
    }
    props.setRailwayMap(railwayMap.setRailwayInfo(id, newRailwayInfo));
  };

  const onOpenStationList = (id: number) => {
    const railwayMap = props.railwayMap;
    if (railwayMap === null) {
      return;
    }
    const stationList = railwayMap.stationListOnRailway(id);
    stationListDialogRef.current!.open(stationList);
  };

  const onDeleteRailway = (id: number) => {
    const railwayMap = props.railwayMap;
    if (railwayMap === null) {
      return;
    }
    props.setRailwayMap(railwayMap.removeRailway(id));
  };

  let cursor = "auto";
  if (state.editorPhase === "none") {
    if (props.editorMode === "borders") {
      const map = props.railwayMap;
      const mouse = state.mouse;
      if (map !== null && mouse !== undefined) {
        const nearest = map.findNearestBorder(
          viewport,
          mouse.x,
          mouse.y,
          distanceThreshold,
        );
        if (nearest !== undefined && "point" in nearest) {
          cursor = "pointer";
        }
      }
    }
  } else if (state.editorPhase === "viewport-moving") {
    cursor = "move";
  } else if (state.editorPhase === "point-moving") {
    cursor = "pointer";
  } else if (state.editorPhase === "station-linking") {
    if (!state.moved) {
      cursor = "auto";
    } else {
      cursor = "pointer";
    }
  } else if (state.editorPhase === "border-moving") {
    cursor = "pointer";
  } else if (state.editorPhase === "border-adding") {
    cursor = "pointer";
  }

  const escapeKeyHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      setState((state) => {
        // reset editorPhase as well as optional states
        return {
          ...state,
          editorPhase: "none",
          mouseXOnMouseDown: undefined,
          mouseYOnMouseDown: undefined,
          topXOnMouseDown: undefined,
          topYOnMouseDown: undefined,
          selectedIndex: undefined,
          selectedBorderIndex: undefined,
          moved: undefined,
        };
      });
    }
  };

  useEffect(() => {
    document.body.addEventListener("keydown", escapeKeyHandler);
    return () => {
      document.body.removeEventListener("keydown", escapeKeyHandler);
    };
  }, []);

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
        onContextMenu={(e) => e.preventDefault()}
      >
        {state.railwayList && (
          <RailwayListViewer
            railwayList={state.railwayList}
            selectedId={state.selectedRailId}
            onSelect={onSelectRailway}
            onOpenRailwayConfig={onOpenRailwayConfig}
            onOpenStationList={onOpenStationList}
            onDeleteRailway={onDeleteRailway}
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
          onContextMenu={(e) => e.preventDefault()}
          style={{
            verticalAlign: "top",
            cursor,
          }}
        />
      </div>
      <StationDialog ref={stationDialogRef}></StationDialog>
      <RailwayDialog ref={railwayDialogRef}></RailwayDialog>
      <StationListDialog ref={stationListDialogRef}></StationListDialog>
    </div>
  );
};
