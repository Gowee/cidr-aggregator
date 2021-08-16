import Typography from "@material-ui/core/Typography";

export default function Header() {
  return (
    <header>
      <Typography variant="h3" component="h1" gutterBottom>
        CIDR Aggregator
      </Typography>
      <Typography variant="h6" component="h2" gutterBottom>
        {"Aggregate or reverse CIDRs (i.e. IP ranges)"}
      </Typography>
    </header>
  );
}
