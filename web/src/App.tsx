import React, { useState } from 'react';
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



import logo from './logo.svg';
import './App.css';

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
    padding: theme.spacing(2)
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
  }
  // footer: {
  //   padding: theme.spacing(3, 2),
  //   marginTop: 'auto',
  //   backgroundColor:
  //     theme.palette.type === 'light' ? theme.palette.grey[200] : theme.palette.grey[800],
  // },
}));

function App() {
  const classes = useStyles();
  const [input, setInput] = useState("");
  const [output, setOutput] = useState({} as any);
  const handleAggregate = async (reverse = false) => {
    const { aggregate } = await import('../../pkg/cidr_aggregator.js');
    setOutput(await aggregate(input, { reverse, ipKind: "Both" }));
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
      {/* <Typography variant="body1">Sticky footer placeholder.</Typography> */}
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
            <Typography variant="caption" color="textSecondary">Lines: | IPv4 Lines: 0 | IPv6 Lines: | Invalid: </Typography>
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
            {/* <Divider orientation="vertical" className={classes.optionsDivider} flexItem /> */}

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
            <Typography variant="caption" color="textSecondary">IPv4 Lines: {(output?.v4?.ranges ?? "").trim().split("\n").length} | IPv4 Addresses: {(output?.v4?.address_count_after ?? "0")} | IPv6 Lines: {(output?.v6?.ranges ?? "").trim().split("\n").length} | IPv6 Addresses: {(output?.v6?.address_count_after ?? "0")} | Invalid Lines: {(output?.invalid ?? "").trim().split("\n").length} </Typography>
          </Box>
        </Paper>
      </main>
      <footer className={classes.footer}>
        <Grid container justifyContent="space-between">
          {/* <Typography variant="body1"></Typography> */}
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
    // </Box>
    // <div className="App">
    //   <header className="App-header">
    //     <img src={logo} className="App-logo" alt="logo" />
    //     <p>
    //       Edit <code>src/App.tsx</code> and save to reload.
    //     </p>
    //     <a
    //       className="App-link"
    //       href="https://reactjs.org"
    //       target="_blank"
    //       rel="noopener noreferrer"
    //     >
    //       Learn React
    //     </a>
    //   </header>
    // </div>
  );
}

export default App;
