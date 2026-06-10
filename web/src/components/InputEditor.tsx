import { useMemo } from "react";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";

import { countLines } from "../utils";
import { statusLineWrapperSx } from "./editorCommon";

export default function InputEditor({
  input,
  setInput,
}: {
  input: string;
  setInput: (value: string) => void;
}) {
  return (
    <>
      <TextField
        id="input"
        label="Input"
        placeholder="6.6.6.0/24"
        multiline
        fullWidth
        autoFocus
        rows={16}
        slotProps={{ htmlInput: { wrap: "soft" } }}
        value={input}
        onChange={(event) => setInput(event.target.value)}
      />
      <Box sx={statusLineWrapperSx}>
        <Typography variant="caption" color="textSecondary">
          Lines: {useMemo(() => countLines(input), [input])}{" "}
        </Typography>
      </Box>
    </>
  );
}
