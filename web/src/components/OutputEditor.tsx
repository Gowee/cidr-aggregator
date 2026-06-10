import Box from "@mui/material/Box";
import TextField from "@mui/material/TextField";

import OutputStatusLine from "./OutputStatusLine";
import WarningBanner from "./WarningFab";

export default function OutputEditor({
  ipKind,
  output,
}: {
  ipKind: string;
  output: any;
}) {
  const hasInvalid = output?.invalid && output.invalid.trim().length > 0;

  return (
    <Box>
      <TextField
        id="input"
        label="Output"
        placeholder="No input"
        variant="standard"
        multiline
        fullWidth
        rows={16}
        error={hasInvalid}
        slotProps={{ htmlInput: { wrap: "soft" } }}
        value={[
          ipKind !== "ipv6" && output?.v4?.ranges,
          ipKind !== "ipv4" && output?.v6?.ranges,
        ]
          .filter((v) => v)
          .join("\n")}
      />
      <WarningBanner invalidLines={output?.invalid} />
      <Box sx={{ mt: 0.5, mb: -1 }}>
        <OutputStatusLine output={output} />
      </Box>
    </Box>
  );
}
