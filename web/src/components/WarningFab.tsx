import React, { useMemo, MouseEvent } from "react";
import Box from "@mui/material/Box";
import Fab from "@mui/material/Fab";
import Tooltip from "@mui/material/Tooltip";
import Popover from "@mui/material/Popover";
import Badge from "@mui/material/Badge";
import WarningIcon from "@mui/icons-material/Warning";
import Typography from "@mui/material/Typography";
import Paper from "@mui/material/Paper";
import Alert from "@mui/material/Alert";
import { orange, red } from "@mui/material/colors";

import { countLines } from "../utils";

export default function WarningFab({ invalidLines }: { invalidLines: string }) {
  const [anchorEl, setAnchorEl] = React.useState<HTMLElement | null>(null);
  const handleOpen = (event: MouseEvent) => {
    setAnchorEl(event.currentTarget as HTMLElement);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };
  const open = Boolean(anchorEl);
  const id = open ? "invalid-lines-popover" : undefined;

  const invalidCount = useMemo(() => countLines(invalidLines), [invalidLines]);

  if (invalidCount === 0) return null;

  const preview = invalidLines.split("\n").filter(Boolean).slice(0, 3);
  const hasMore = invalidCount > 3;

  return (
    <>
      {/* Inline warning banner */}
      <Alert
        severity="warning"
        icon={<WarningIcon />}
        sx={{
          mt: 1,
          mb: 1,
          cursor: "pointer",
          "& .MuiAlert-message": { width: "100%" },
        }}
        onClick={handleOpen}
      >
        <Typography variant="body2" sx={{ fontWeight: "bold" }}>
          {invalidCount} invalid line{invalidCount !== 1 && "s"} skipped
        </Typography>
        <Box component="pre" sx={{ m: 0, mt: 0.5, fontSize: "0.8rem", whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
          {preview.join("\n")}
          {hasMore && `\n… and ${invalidCount - 3} more`}
        </Box>
      </Alert>

      {/* Floating Fab for details popover */}
      <Tooltip title="Show all invalid lines">
        <Fab
          size="small"
          sx={{
            position: "absolute",
            right: (theme) => theme.spacing(3),
            bottom: (theme) => theme.spacing(5),
            color: "white",
            backgroundColor: red[600],
            animation: "pulse 2s ease-in-out infinite",
            "@keyframes pulse": {
              "0%": { boxShadow: `0 0 0 0 ${red[600]}80` },
              "70%": { boxShadow: `0 0 0 10px ${red[600]}00` },
              "100%": { boxShadow: `0 0 0 0 ${red[600]}00` },
            },
            "&:hover": { backgroundColor: red[700] },
          }}
          aria-label="Show warnings"
          onClick={handleOpen}
        >
          <Badge badgeContent={invalidCount} color="warning">
            <WarningIcon />
          </Badge>
        </Fab>
      </Tooltip>

      <Popover
        id={id}
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "center",
        }}
      >
        <Box sx={{ m: 1, maxWidth: "40em" }}>
          <Typography variant="h6" sx={{ textAlign: "center" }}>
            Invalid lines
          </Typography>
          <Paper variant="outlined" square>
            <pre style={{ padding: 8, margin: 0, minWidth: "12em", maxHeight: "20em", overflow: "auto" }}>
              <code>{invalidLines}</code>
            </pre>
          </Paper>
        </Box>
      </Popover>
    </>
  );
}
