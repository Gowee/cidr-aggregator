import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";

function Partial({ name, status }: { name: string; status: any }) {
  return (
    <Typography variant="caption" color="textSecondary">
      {name}: {status?.line_count_before ?? 0} <abbr title="Lines">L</abbr> /{" "}
      {status?.address_count_before ?? "0"}
      &nbsp;&nbsp;➟&nbsp;&nbsp;
      <b>{status?.line_count_after ?? 0}</b>
      <abbr title="Lines">L</abbr> / <b>{status?.address_count_after ?? "0"}</b>
    </Typography>
  );
}

export default function OutputStatusLine({ output }: { output: any }) {
  return (
    <Grid container sx={{ justifyContent: "space-between" }}>
      <Grid>
        <Partial name="IPv4" status={output?.v4} />
      </Grid>
      <Grid>
        <Partial name="IPv6" status={output?.v6} />
      </Grid>
    </Grid>
  );
}
