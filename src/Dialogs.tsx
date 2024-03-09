import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControlLabel,
  Radio,
  RadioGroup,
  TextField,
} from "@mui/material";
import { ChangeEvent, forwardRef, useImperativeHandle, useState } from "react";
import { RailwayInfo, StationInfo, StationListOnRailway } from "./RerailMap";
import { MuiColorInput } from "mui-color-input";

type StationDialogState = {
  open: boolean;
  value: StationInfo;
  callback?: (value?: StationInfo) => void;
};

export type StationDialogRefType = {
  open: (initialValue: StationInfo) => Promise<StationInfo | undefined>;
};

export const StationDialog = forwardRef((_props, ref) => {
  const [state, setState] = useState<StationDialogState>({
    open: false,
    value: {
      name: "",
      level: 0,
    },
  });

  useImperativeHandle(
    ref,
    () => {
      return {
        open(initialValue: StationInfo): Promise<StationInfo | undefined> {
          return new Promise((resolve: (value?: StationInfo) => void) => {
            setState({
              open: true,
              value: initialValue,
              callback: resolve,
            });
          });
        },
      };
    },
    [],
  );

  const onClick = (ok: boolean) => {
    state.callback!(ok ? state.value : undefined);
    setState({ open: false, value: state.value, callback: undefined });
  };

  const onChangeName = (e: ChangeEvent<HTMLInputElement>) => {
    setState({
      ...state,
      value: {
        ...state.value,
        name: e.target.value,
      },
    });
  };

  const onChangeLevel = (level: number) => {
    setState({
      ...state,
      value: {
        ...state.value,
        level,
      },
    });
  };

  return (
    <Dialog open={state.open}>
      <DialogTitle>駅設定</DialogTitle>
      <DialogContent>
        <TextField
          label="駅名"
          margin="dense"
          value={state.value.name}
          onChange={onChangeName}
        />
        <RadioGroup value={state.value.level}>
          <FormControlLabel
            value={0}
            control={<Radio />}
            onChange={() => onChangeLevel(0)}
            label="普通駅"
          />
          <FormControlLabel
            value={1}
            control={<Radio />}
            onChange={() => onChangeLevel(1)}
            label="主要駅"
          />
          <FormControlLabel
            value={2}
            control={<Radio />}
            onChange={() => onChangeLevel(2)}
            label="地域代表駅"
          />
          <FormControlLabel
            value={3}
            control={<Radio />}
            onChange={() => onChangeLevel(3)}
            label="都市代表駅"
          />
        </RadioGroup>
      </DialogContent>
      <DialogActions>
        <Button autoFocus onClick={() => onClick(false)}>
          キャンセル
        </Button>
        <Button onClick={() => onClick(true)}>OK</Button>
      </DialogActions>
    </Dialog>
  );
});

type RailwayDialogState = {
  open: boolean;
  value: RailwayInfo;
  callback?: (value?: RailwayInfo) => void;
};

export type RailwayDialogRefType = {
  open: (initialValue: RailwayInfo) => Promise<RailwayInfo | undefined>;
};

export const RailwayDialog = forwardRef((_props, ref) => {
  const [state, setState] = useState<RailwayDialogState>({
    open: false,
    value: {
      name: "",
      color: 0x000000,
      level: 0,
    },
  });

  useImperativeHandle(
    ref,
    () => {
      return {
        open(initialValue: RailwayInfo): Promise<RailwayInfo | undefined> {
          return new Promise((resolve: (value?: RailwayInfo) => void) => {
            setState({
              open: true,
              value: initialValue,
              callback: resolve,
            });
          });
        },
      };
    },
    [],
  );

  const onClick = (ok: boolean) => {
    state.callback!(ok ? state.value : undefined);
    setState({ open: false, value: state.value, callback: undefined });
  };

  const onChangeName = (e: ChangeEvent<HTMLInputElement>) => {
    setState({
      ...state,
      value: {
        ...state.value,
        name: e.target.value,
      },
    });
  };

  const onChangeLevel = (level: number) => {
    setState({
      ...state,
      value: {
        ...state.value,
        level,
      },
    });
  };

  const onChangeColor = (color: string) => {
    const r = parseInt(color.slice(1, 3), 16);
    const g = parseInt(color.slice(3, 5), 16);
    const b = parseInt(color.slice(5, 7), 16);
    setState({
      ...state,
      value: {
        ...state.value,
        color: (r << 16) | (g << 8) | (b << 0),
      },
    });
  };

  const color = "#" + ("000000" + state.value.color.toString(16)).slice(-6);

  return (
    <Dialog open={state.open}>
      <DialogTitle>路線設定</DialogTitle>
      <DialogContent>
        <div>
          <TextField
            label="路線名"
            margin="dense"
            value={state.value.name}
            onChange={onChangeName}
          />
        </div>
        <div>
          <MuiColorInput
            format="hex"
            value={color}
            onChange={onChangeColor}
            isAlphaHidden
          />
        </div>
        <RadioGroup value={state.value.level}>
          <FormControlLabel
            value={0}
            control={<Radio />}
            onChange={() => onChangeLevel(0)}
            label="地下鉄"
          />
          <FormControlLabel
            value={1}
            control={<Radio />}
            onChange={() => onChangeLevel(1)}
            label="地域輸送路線"
          />
          <FormControlLabel
            value={2}
            control={<Radio />}
            onChange={() => onChangeLevel(2)}
            label="広域輸送路線"
          />
          <FormControlLabel
            value={3}
            control={<Radio />}
            onChange={() => onChangeLevel(3)}
            label="超広域輸送路線"
          />
        </RadioGroup>
      </DialogContent>
      <DialogActions>
        <Button autoFocus onClick={() => onClick(false)}>
          キャンセル
        </Button>
        <Button onClick={() => onClick(true)}>OK</Button>
      </DialogActions>
    </Dialog>
  );
});

export type StationListDialogRefType = {
  open: (initialValue: StationListOnRailway) => void;
};

export const StationListDialog = forwardRef((_props, ref) => {
  const [stationList, setStationList] = useState<StationListOnRailway | null>(
    null,
  );

  useImperativeHandle(
    ref,
    () => {
      return {
        open(s: StationListOnRailway) {
          setStationList(s);
        },
      };
    },
    [],
  );

  let rows = [];
  if (stationList !== null) {
    const names = stationList.names;
    const distances = stationList.distances;
    for (let i = 0; i < names.length; ++i) {
      rows.push(
        <tr>
          <td style={{ width: "70%", zIndex: 2 }}>{names[i]}</td>
          <td style={{ width: "30%", zIndex: 2 }}>
            {(distances[i] / 1000).toFixed(2)}
          </td>
        </tr>,
      );
    }
  }

  return (
    <Dialog open={stationList !== null} fullWidth>
      <DialogTitle>駅一覧</DialogTitle>
      <DialogContent style={{ height: "100%" }}>
        <table style={{ width: "100%" }} cellSpacing={0} border={1}>
          <thead style={{ position: "sticky" }}>
            <tr>
              <th
                style={{
                  width: "70%",
                  position: "sticky",
                  top: 0,
                  zIndex: 1,
                  whiteSpace: "nowrap",
                  backgroundColor: "#eeeeee",
                }}
              >
                駅名
              </th>
              <th
                style={{
                  width: "30%",
                  position: "sticky",
                  top: 0,
                  zIndex: 1,
                  whiteSpace: "nowrap",
                  backgroundColor: "#eeeeee",
                }}
              >
                距離(km)
              </th>
            </tr>
          </thead>
          {rows}
        </table>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setStationList(null)}>閉じる</Button>
      </DialogActions>
    </Dialog>
  );
});
