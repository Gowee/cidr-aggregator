import React, { useMemo } from "react";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import Alert from "@mui/material/Alert";
import WarningIcon from "@mui/icons-material/Warning";

import { countLines } from "../utils";

export default function WarningBanner({ invalidLines }: { invalidLines: string }) {
  const invalidCount = useMemo(() => countLines(invalidLines), [invalidLines]);

  if (invalidCount === 0) return null;

  return (
    <Alert variant="standard" severity="warning" icon={<WarningIcon />} sx={{ mt: 1, mb: 1 }}>
      <Typography variant="body2" sx={{ fontWeight: "bold" }}>
        {invalidCount} invalid line{invalidCount !== 1 && "s"} skipped
      </Typography>
      <TextField
        variant="standard"
        multiline
        fullWidth
        value={invalidLines}
        onFocus={(event) => event.target.select()}
        sx={{
          mt: 0.5,
          "& .MuiInput-underline:before": { borderBottom: "none" },
          "& .MuiInput-underline:after": { borderBottom: "none" },
          "& .MuiInput-underline:hover:not(.Mui-disabled):before": { borderBottom: "none" },
        }}
        slotProps={{
          input: { readOnly: true },
          htmlInput: { style: { fontSize: "0.8rem", padding: 4 } },
        }}
      />
    </Alert>
  );
}
