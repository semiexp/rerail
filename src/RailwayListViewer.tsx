import React, { useState, useRef } from "react";
import { ConfirmationDialog, ConfirmationDialogRefType } from "./Dialogs";
import { ViewportRailwayList } from "./RerailMap";
import { Menu, MenuItem } from "@mui/material";

type RailwayListViewerProps = {
  railwayList: ViewportRailwayList;
  selectedId: number | null;
  onSelect: (id: number) => void;
  onOpenRailwayConfig: (id: number) => void;
  onOpenStationList: (id: number) => void;
  onDeleteRailway: (id: number) => void;
};

export const RailwayListViewer = (props: RailwayListViewerProps) => {
  const { railwayList, selectedId, onSelect } = props;
  const railNames = railwayList.rail_names;
  const railIds = railwayList.rail_ids;

  const [contextMenu, setContextMenu] = useState<{
    mouseX: number;
    mouseY: number;
    selectedRail: number;
  } | null>(null);

  const confirmationDialogRef = useRef<ConfirmationDialogRefType>(null);

  let items = [];
  for (let i = 0; i < railNames.length; ++i) {
    let extraStyle = {};
    if (railIds[i] === selectedId) {
      extraStyle = { backgroundColor: "#ddddff" };
    }
    items.push(
      <div
        className="railwayListItem"
        style={{
          width: "100%",
          textOverflow: "ellipsis",
          overflow: "hidden",
          whiteSpace: "nowrap",
          ...extraStyle,
        }}
        onClick={() => onSelect(railIds[i])}
        onContextMenu={(e: React.MouseEvent) => {
          e.preventDefault();
          if (contextMenu !== null) {
            setContextMenu(null);
          } else {
            setContextMenu({
              mouseX: e.clientX,
              mouseY: e.clientY,
              selectedRail: railIds[i],
            });
          }
        }}
      >
        {railNames[i]}
      </div>,
    );
  }

  return (
    <div
      style={{ height: "100%", width: "100%", overflowY: "scroll" }}
      onContextMenu={(e) => e.preventDefault()}
    >
      {items}
      <Menu
        open={contextMenu !== null}
        anchorReference="anchorPosition"
        anchorPosition={
          contextMenu
            ? { top: contextMenu.mouseY, left: contextMenu.mouseX }
            : undefined
        }
        onClose={() => setContextMenu(null)}
      >
        <MenuItem
          onClick={() => {
            if (contextMenu !== null) {
              props.onOpenRailwayConfig(contextMenu.selectedRail);
            }
            setContextMenu(null);
          }}
        >
          路線設定
        </MenuItem>
        <MenuItem
          onClick={() => {
            if (contextMenu !== null) {
              props.onOpenStationList(contextMenu.selectedRail);
            }
            setContextMenu(null);
          }}
        >
          駅一覧
        </MenuItem>
        <MenuItem
          onClick={async () => {
            if (contextMenu !== null) {
              setContextMenu(null);
              const result =
                await confirmationDialogRef.current!.open(
                  "路線を削除しますか？",
                );
              if (result) {
                props.onDeleteRailway(contextMenu.selectedRail);
              }
            }
          }}
        >
          路線を削除
        </MenuItem>
      </Menu>
      <ConfirmationDialog ref={confirmationDialogRef} />
    </div>
  );
};
