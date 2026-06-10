import React, { useMemo, MouseEvent } from "react";
import Box from "@mui/material/Box";
import Fab from "@mui/material/Fab";
import Tooltip from "@mui/material/Tooltip";
import Popover from "@mui/material/Popover";
import Badge from "@mui/material/Badge";
import WarningIcon from "@mui/icons-material/Warning";
import Typography from "@mui/material/Typography";
import Paper from "@mui/material/Paper";
import { orange } from "@mui/material/colors";

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

  return invalidCount > 0 ? (
    <>
      <Tooltip title={invalidCount + " invalid lines"}>
        <Fab
          size="small"
          sx={{
            position: "absolute",
            right: (theme) => theme.spacing(3),
            bottom: (theme) => theme.spacing(5),
            color: orange[500],
            backgroundColor: "white",
            borderColor: (theme) => theme.palette.primary.light,
          }}
          aria-label="Show warnings"
          onClick={handleOpen}
        >
          <Badge badgeContent={invalidCount} color="secondary">
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
        <Box sx={{ m: 1 }}>
          <Typography variant="h6" sx={{ textAlign: "center" }}>
            Invalid lines
          </Typography>
          <Paper variant="outlined" square>
            <pre style={{ padding: 8, margin: 0, minWidth: "12em" }}>
              <code>{invalidLines}</code>
            </pre>
          </Paper>
        </Box>
      </Popover>
    </>
  ) : (
    <></>
  );
}
