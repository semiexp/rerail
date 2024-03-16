import iconThinBorder from "./assets/thinBorder.svg";
import iconDottedBorder from "./assets/dottedBorder.svg";
import iconBoldBorder from "./assets/boldBorder.svg";
import { ToggleButton } from "@mui/material";

export type BorderSelecterProps = {
  value: number;
  onChange: (value: number) => void;
};

export const BorderSelecter = (props: BorderSelecterProps) => {
  const value = props.value;

  return (
    <ToggleButton
      value="borderSelecter"
      sx={{ padding: 0.2 }}
      onClick={() => props.onChange((value + 1) % 3)}
    >
      <img
        src={
          value === 0
            ? iconDottedBorder
            : value === 1
              ? iconThinBorder
              : iconBoldBorder
        }
        height={24}
      />
    </ToggleButton>
  );
};
