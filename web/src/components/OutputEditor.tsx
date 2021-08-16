import Box from "@material-ui/core/Box";
// import Paper from "@material-ui/core/Paper";
import TextField from "@material-ui/core/TextField";

import { useEditorStyles } from "./editorCommon";
import OutputStatusLine from "./OutputStatusLine";
import WarningFab from "./WarningFab";

export default function OutputEditor({
  ipKind,
  output,
}: {
  ipKind: string;
  output: any;
}) {
  const classes = useEditorStyles();

  return (
    <Box position="relative">
      <TextField
        id="input"
        label="Output"
        placeholder="No input"
        multiline
        fullWidth
        rows={16}
        value={[
          ipKind !== "ipv6" && output?.v4?.ranges,
          ipKind !== "ipv4" && output?.v6?.ranges,
        ]
          .filter((v) => v)
          .join("\n")}
      />
      <Box className={classes.statusLineWrapper}>
        <OutputStatusLine output={output} />
      </Box>
      <WarningFab invalidLines={output?.invalid} />
    </Box>
  );
}
