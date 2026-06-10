import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
import Link from "@mui/material/Link";
import GitHubIcon from "@mui/icons-material/GitHub";

export default function Footer() {
  return (
    <footer style={{ marginTop: 24 }}>
      <Grid container sx={{ justifyContent: "space-between" }}>
        <Grid>
          <Typography variant="body2" color="textSecondary">
            <Link
              color="inherit"
              href="https://github.com/Gowee/cidr-aggregator"
            >
              <GitHubIcon sx={{ fontSize: "1rem" }} />
              {" Source code"}
            </Link>
          </Typography>
        </Grid>
        <Grid>
          <Typography variant="body2" color="textSecondary">
            {"The tool works fully in the browser and has no data collected."}
          </Typography>
        </Grid>
      </Grid>
    </footer>
  );
}
