import React, { useMemo } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import Alert from "@mui/material/Alert";
import WarningIcon from "@mui/icons-material/Warning";

import { countLines } from "../utils";

export default function WarningBanner({ invalidLines }: { invalidLines: string }) {
  const invalidCount = useMemo(() => countLines(invalidLines), [invalidLines]);

  if (invalidCount === 0) return null;

  const preview = invalidLines.split("\n").filter(Boolean).slice(0, 3);
  const hasMore = invalidCount > 3;

  return (
    <Alert variant="standard" severity="warning" icon={<WarningIcon />} sx={{ mt: 1, mb: 1 }}>
      <Typography variant="body2" sx={{ fontWeight: "bold" }}>
        {invalidCount} invalid line{invalidCount !== 1 && "s"} skipped
      </Typography>
      <Box component="pre" sx={{ m: 0, mt: 0.5, fontSize: "0.8rem", whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
        {preview.join("\n")}
        {hasMore && `\n… and ${invalidCount - 3} more`}
      </Box>
    </Alert>
  );
}
