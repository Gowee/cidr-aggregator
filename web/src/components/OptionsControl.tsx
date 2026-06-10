import { forwardRef, ForwardedRef } from "react";

import FormGroup from "@mui/material/FormGroup";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
import ButtonGroup from "@mui/material/ButtonGroup";
import Button from "@mui/material/Button";
import Tooltip from "@mui/material/Tooltip";

function OptionsControl(
  {
    ipKind,
    toggleIpv4,
    toggleIpv6,
    bogonFilter,
    toggleReservedFilter,
    handleAggregate,
  }: {
    ipKind: string;
    toggleIpv4: () => void;
    toggleIpv6: () => void;
    bogonFilter?: string;
    toggleReservedFilter: () => void;
    handleAggregate: (reverse?: boolean) => void;
  },
  ref: ForwardedRef<any>
) {
  return (
    <Grid container ref={ref} sx={{ justifyContent: "space-around" }}>
      <Grid>
        <ButtonGroup color="primary" aria-label="control button group">
          <Button color="primary" onClick={() => handleAggregate()}>
            Aggregate
          </Button>
          <Button color="primary" onClick={() => handleAggregate(true)}>
            Reverse
          </Button>
        </ButtonGroup>
      </Grid>
      <Grid>
        <FormGroup row>
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv6"}
                onChange={toggleIpv4}
                name="ipv4"
              />
            }
            label="IPv4"
          />
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv4"}
                onChange={toggleIpv6}
                name="ipv6"
              />
            }
            label="IPv6"
          />
        </FormGroup>
      </Grid>
      <Grid>
        <Tooltip title="If activated, all reserved IPs for special purposes (RFC 5735 and RFC 6890) are filtered out from the output. This might be useful when reversing.">
          <FormControlLabel
            control={
              <Switch
                checked={bogonFilter === "reserved"}
                onChange={toggleReservedFilter}
                name="excludeReserved"
              />
            }
            label="Exclude reserved IPs"
          />
        </Tooltip>
      </Grid>
    </Grid>
  );
}

export default forwardRef(OptionsControl);
