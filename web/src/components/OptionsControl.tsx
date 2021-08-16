import { forwardRef, ForwardedRef } from "react";

import FormGroup from "@material-ui/core/FormGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Switch from "@material-ui/core/Switch";
import Grid from "@material-ui/core/Grid";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import Button from "@material-ui/core/Button";

function OptionsControl(
  {
    ipKind,
    toggleIpv4,
    toggleIpv6,
    handleAggregate,
  }: {
    ipKind: string;
    toggleIpv4: () => void;
    toggleIpv6: () => void;
    handleAggregate: (reverse?: boolean) => void;
  },
  ref: ForwardedRef<any>
) {
  return (
    <Grid container ref={ref} direction="row" justifyContent="space-around">
      <Grid item>
        <ButtonGroup color="primary" aria-label="control button group">
          <Button color="primary" onClick={() => handleAggregate()}>
            Aggregate
          </Button>
          <Button color="primary" onClick={() => handleAggregate(true)}>
            Reverse
          </Button>
        </ButtonGroup>
      </Grid>
      <Grid item>
        <FormGroup row>
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv6"}
                onChange={toggleIpv4}
                name="checkedA"
              />
            }
            label="IPv4"
          />
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv4"}
                onChange={toggleIpv6}
                name="checkedB"
              />
            }
            label="IPv6"
          />
        </FormGroup>
      </Grid>
    </Grid>
  );
}

export default forwardRef(OptionsControl);
