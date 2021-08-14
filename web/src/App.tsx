import React, { useState, useMemo, MouseEvent } from 'react';
import CssBaseline from '@material-ui/core/CssBaseline';
import Typography from '@material-ui/core/Typography';
import { makeStyles } from '@material-ui/core/styles';
import Container from '@material-ui/core/Container';
import Link from '@material-ui/core/Link';
import Box from '@material-ui/core/Box';
import TextField from '@material-ui/core/TextField';
import Paper from '@material-ui/core/Paper';
import Button from '@material-ui/core/Button';
import ButtonGroup from '@material-ui/core/ButtonGroup';
import Divider from '@material-ui/core/Divider';
import Grid from '@material-ui/core/Grid';
import FormGroup from '@material-ui/core/FormGroup';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import Switch from '@material-ui/core/Switch';
import GitHubIcon from '@material-ui/icons/GitHub';
import Popover from '@material-ui/core/Popover';
import IconButton from '@material-ui/core/IconButton';
import Badge from '@material-ui/core/Badge';
import WarningIcon from '@material-ui/icons/Warning';
import Fab from '@material-ui/core/Fab';
import Tooltip from '@material-ui/core/Tooltip';
import orange from '@material-ui/core/colors/orange';
// import ShoppingCartIcon from '@material-ui/core/ShoppingCartIcon';

import logo from './logo.svg';
import './App.css';

import { count_lines } from './utils';

const useStyles = makeStyles((theme) => ({
  // root: {
  //   display: 'flex',
  //   flexDirection: 'column',
  //   minHeight: '100vh',
  // },
  main: {
    marginTop: theme.spacing(5),
    marginBottom: theme.spacing(2),
  },
  editorWrapper: {
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    padding: theme.spacing(2),
    position: "relative" // for Fab positioning
  },
  editorStatus: {
    marginTop: theme.spacing(0.5),
    marginBottom: theme.spacing(-1),
  },
  optionsWrapper: {
    marginTop: theme.spacing(-1),
    marginBottom: theme.spacing(-1),
    padding: theme.spacing(1),
    // '& > *': {
    //   margin: theme.spacing(1)
    // }
  },
  optionsDivider: {
    marginLeft: theme.spacing(2),
    marginRight: theme.spacing(2)
  },
  footer: {
    marginTop: theme.spacing(3)
  },
  warningFab: {
    position: "absolute",
    right: theme.spacing(3),
    bottom: theme.spacing(5),
    color: orange[500],
    backgroundColor: "white",
    borderColor: theme.palette.primary.light,
    // "&:hover": {
    //   backgroundColor: "white",
    // }
  },
  fabWrapper: {
    // position: "relative"
  }
  // footer: {
  //   padding: theme.spacing(3, 2),
  //   marginTop: 'auto',
  //   backgroundColor:
  //     theme.palette.type === 'light' ? theme.palette.grey[200] : theme.palette.grey[800],
  // },
}));

function OutputStatus({ output, classes }: { output: any, classes: any }) {
  console.log(output);
  let [showInvalid, setShowInvalid] = useState(false);
  const invalid_count = useMemo(() => count_lines(output?.invalid), [output?.invalid]);
  return (
    <Grid container direction="row" justifyContent="space-between">
      <Grid item>
        <Typography variant="caption" color="textSecondary">
          IPv4: {output?.v4?.line_count_before ?? 0}<abbr title="Lines">L</abbr> / {(output?.v4?.address_count_before ?? "0")}
          &nbsp;&nbsp;➟&nbsp;&nbsp;
          <b>{output?.v4?.line_count_after ?? 0}</b><abbr title="Lines">L</abbr> / <b>{(output?.v4?.address_count_after ?? "0")}</b>
        </Typography>
      </Grid>
      <Grid item>
        <Typography variant="caption" color="textSecondary">
          IPv6: {output?.v6?.line_count_before ?? 0}<abbr title="Lines">L</abbr> / {(output?.v6?.address_count_before ?? "0")}
          &nbsp;&nbsp;➟&nbsp;&nbsp;
          <b>{output?.v6?.line_count_after ?? 0}</b><abbr title="Lines">L</abbr> / <b>{(output?.v6?.address_count_after ?? "0")}</b>
        </Typography>
      </Grid>
    </Grid>);
}

function WarningFab({ className, invalidLines }: { className: string, invalidLines: string }) {
  const invalid_count = useMemo(() => count_lines(invalidLines), [invalidLines]);

  const [anchorEl, setAnchorEl] = React.useState(null as any);
  const handleOpen = (event: MouseEvent) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };
  const open = Boolean(anchorEl);
  const id = open ? 'invalid-lines-popover' : undefined;

  return invalid_count > 0 ? (
    <>
      <Tooltip title={invalid_count + " invalid lines"}>
        <Fab size="small" className={className} aria-label="Show warnings" onClick={handleOpen}>
          <Badge badgeContent={invalid_count} color="secondary">
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
          vertical: 'bottom',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'center',
        }}
      >
        <Typography variant="h6">Invalid lines</Typography>
        <pre>
          <code>
            {invalidLines}
          </code>
        </pre>
      </Popover>
    </>
  ) : (<></>);
}

function App() {
  const classes = useStyles();
  const [input, setInput] = useState("");
  const [output, setOutput] = useState({} as any);
  const handleAggregate = async (reverse = false) => {
    const { aggregate } = await import('../../pkg/cidr_aggregator.js');
    setOutput(await aggregate(input, reverse));
  };
  console.log(output);
  console.log((output?.V4?.ranges));
  return (
    // <Box display="flex">
    <Container component="main" className={classes.main} maxWidth="md">
      <CssBaseline />
      <header>
        <Typography variant="h3" component="h1" gutterBottom>
          CIDR Aggregator
        </Typography>
        <Typography variant="h6" component="h2" gutterBottom>
          {'Aggregate or reverse CIDRs (i.e. IP ranges)'}
        </Typography>
      </header>
      <main>
        <Paper elevation={1} className={classes.editorWrapper}>
          <TextField
            id="input"
            label="Input"
            placeholder="6.6.6.0/24"
            multiline
            fullWidth
            rows={16}
            value={input}
            onChange={(event) => setInput(event.target.value)}
          />
          <Box className={classes.editorStatus}>
            <Typography variant="caption" color="textSecondary">Lines: {useMemo(() => count_lines(input), [input])} </Typography>
          </Box>
        </Paper>
        <Paper elevation={1} className={classes.optionsWrapper}>
          <Grid
            container
            direction="row"
            justifyContent="space-around"
          >
            <Grid item>
              <ButtonGroup color="primary" aria-label="control button group">
                <Button color="primary" onClick={() => handleAggregate()}>Aggregate</Button>
                <Button color="primary" onClick={() => handleAggregate(true)}>Reverse</Button>
              </ButtonGroup>
            </Grid>
            <Grid item>
              <FormGroup row>
                <FormControlLabel
                  control={<Switch /*checked={state.checkedA} onChange={handleChange}*/ name="checkedA" checked />}
                  label="IPv4"
                />
                <FormControlLabel
                  control={
                    <Switch
                      // checked={state.checkedB}
                      // onChange={handleChange}
                      name="checkedB"
                      checked
                    />
                  }
                  label="IPv6"
                />
              </FormGroup>
            </Grid>
          </Grid>
        </Paper>
        <Paper elevation={1} className={classes.editorWrapper}>
          <TextField
            id="input"
            label="Output"
            placeholder="No input"
            multiline
            fullWidth
            rows={16}
            value={[output?.v4?.ranges, output?.v6?.ranges].filter((v) => v).join("\n")}
          />
          <Box className={classes.editorStatus}>
            <OutputStatus output={output} classes={classes} />
          </Box>
          <WarningFab className={classes.warningFab} invalidLines={output?.invalid} />
        </Paper>
      </main>
      <footer className={classes.footer}>
        <Grid container justifyContent="space-between">
          <Grid item>
            <Typography variant="body2" color="textSecondary">
              <Link color="inherit" href="https://github.com/Gowee/cidr-aggregator">
                <GitHubIcon style={{ fontSize: "1rem" }} />
                {" Source code"}
              </Link>
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="body2" color="textSecondary">
              {'The tool works fully in the browser and has no data collected.'}
            </Typography>
          </Grid>
        </Grid>
      </footer>
    </Container>
  );
}

export default App;
