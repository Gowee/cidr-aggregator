import { useMemo } from "react";
import TextField from "@material-ui/core/TextField";
// import Paper from "@material-ui/core/Paper";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";

import { countLines } from "../utils";
import { useEditorStyles } from "./editorCommon";

export default function InputEditor({
  input,
  setInput,
}: {
  input: string;
  setInput: (value: string) => void;
}) {
  const classes = useEditorStyles();

  return (
    <>
      <TextField
        id="input"
        label="Input"
        placeholder="6.6.6.0/24"
        multiline
        fullWidth
        rows={16}
        value={input}
        onChange={(event) => setInput(event.target.value)}
      />
      <Box className={classes.statusLineWrapper}>
        <Typography variant="caption" color="textSecondary">
          Lines: {useMemo(() => countLines(input), [input])}{" "}
        </Typography>
      </Box>
    </>
  );
}
