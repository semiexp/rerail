import { ViewportRailwayList } from "../rerail-internal/pkg/rerail_internal";

type RailwayListViewerProps = {
  railwayList: ViewportRailwayList;
  selectedId: number | null;
  onSelect: (id: number) => void;
};

export const RailwayListViewer = (props: RailwayListViewerProps) => {
  const { railwayList, selectedId, onSelect } = props;
  const railNames = railwayList.rail_names;
  const railIds = railwayList.rail_ids;

  let items = [];
  for (let i = 0; i < railNames.length; ++i) {
    let extraStyle = {};
    if (railIds[i] === selectedId) {
      extraStyle = { backgroundColor: "#ddddff" };
    }
    items.push(
      <div
        style={{
          width: "100%",
          textOverflow: "ellipsis",
          overflow: "hidden",
          whiteSpace: "nowrap",
          ...extraStyle,
        }}
        onClick={() => onSelect(railIds[i])}
      >
        {railNames[i]}
      </div>,
    );
  }

  return (
    <div style={{ height: "100%", width: "100%", overflowY: "scroll" }}>
      {items}
    </div>
  );
};
