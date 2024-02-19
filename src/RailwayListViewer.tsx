import { ViewportRailwayList } from "../rerail-internal/pkg/rerail_internal";

type RailwayListViewerProps = {
  railwayList: ViewportRailwayList;
};

export const RailwayListViewer = (props: RailwayListViewerProps) => {
  const { railwayList } = props;
  const railNames = railwayList.rail_names;

  let items = [];
  for (let i = 0; i < railNames.length; ++i) {
    items.push(<div>{railNames[i]}</div>);
  }

  return <div style={{ height: "100%", overflowY: "scroll" }}>{items}</div>;
};
