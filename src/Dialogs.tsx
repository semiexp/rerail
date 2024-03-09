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
import { StationInfo } from "./RerailMap";

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
