import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
} from "@mui/material";
import { ChangeEvent, forwardRef, useImperativeHandle, useState } from "react";

export type StationDialogValue = {
  name: string;
};

type StationDialogState = {
  open: boolean;
  value: StationDialogValue;
  callback?: (value?: StationDialogValue) => void;
};

export type StationDialogRefType = {
  open: (
    initialValue: StationDialogValue,
  ) => Promise<StationDialogValue | undefined>;
};

export const StationDialog = forwardRef((_props, ref) => {
  const [state, setState] = useState<StationDialogState>({
    open: false,
    value: {
      name: "",
    },
  });

  useImperativeHandle(
    ref,
    () => {
      return {
        open(
          initialValue: StationDialogValue,
        ): Promise<StationDialogValue | undefined> {
          return new Promise(
            (resolve: (value?: StationDialogValue) => void) => {
              setState({
                open: true,
                value: initialValue,
                callback: resolve,
              });
            },
          );
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
