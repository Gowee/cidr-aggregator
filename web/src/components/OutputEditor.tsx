import Box from "@mui/material/Box";
import TextField from "@mui/material/TextField";

import OutputStatusLine from "./OutputStatusLine";
import WarningFab from "./WarningFab";

export default function OutputEditor({
  ipKind,
  output,
}: {
  ipKind: string;
  output: any;
}) {
  return (
    <Box sx={{ position: "relative" }}>
      {" "}
      {/* for Fab positioning */}
      {/* TODO: nowrap */}
      <TextField
        id="input"
        label="Output"
        placeholder="No input"
        multiline
        fullWidth
        rows={16}
        slotProps={{ htmlInput: { wrap: "soft" } }}
        value={[
          ipKind !== "ipv6" && output?.v4?.ranges,
          ipKind !== "ipv4" && output?.v6?.ranges,
        ]
          .filter((v) => v)
          .join("\n")}
      />
      <WarningFab invalidLines={output?.invalid} />
      <Box sx={{ mt: 0.5, mb: -1 }}>
        <OutputStatusLine output={output} />
      </Box>
    </Box>
  );
}
