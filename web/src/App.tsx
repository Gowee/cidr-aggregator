import React, { useState, useRef } from "react";
import CssBaseline from "@material-ui/core/CssBaseline";
import { makeStyles } from "@material-ui/core/styles";
import Container from "@material-ui/core/Container";
import Paper from "@material-ui/core/Paper";
import Box from "@material-ui/core/Box";

// import logo from './logo.svg';
// import './App.css';

import Header from "./components/Header";
import Footer from "./components/Footer";
import InputEditor from "./components/InputEditor";
import OutputEditor from "./components/OutputEditor";
import OptionsControl from "./components/OptionsControl";

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
    position: "relative", // for Fab positioning
  },
  optionsControlWrapper: {
    marginTop: theme.spacing(-1),
    marginBottom: theme.spacing(-1),
    padding: theme.spacing(1),
    // '& > *': {
    //   margin: theme.spacing(1)
    // }
  },
  mainWrapper: {
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(1),
  },
  sectionWrapper: {
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(1),
    // padding: theme.spacing(2),
  },
  // footer: {
  //   padding: theme.spacing(3, 2),
  //   marginTop: 'auto',
  //   backgroundColor:
  //     theme.palette.type === 'light' ? theme.palette.grey[200] : theme.palette.grey[800],
  // },
}));

function App() {
  const classes = useStyles();
  const controlRef = useRef(null as any);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState({} as any);
  const [ipKind, setIpKind] = useState("both");
  const toggleIpv4 = () => {
    setIpKind((prev) => {
      if (prev === "both" || prev === "ipv4") {
        return "ipv6";
      } /* ipv6 */ else {
        return "both";
      }
    });
  };
  const toggleIpv6 = () => {
    setIpKind((prev) => {
      if (prev === "both" || prev === "ipv6") {
        return "ipv4";
      } /* ipv4 */ else {
        return "both";
      }
    });
  };
  const handleAggregate = async (reverse = false) => {
    const { aggregate } = await import("../../pkg/cidr_aggregator.js");
    setOutput(await aggregate(input, reverse));
    controlRef?.current &&
      controlRef.current.scrollIntoView({ behavior: "smooth" });
  };

  return (
    // <Box display="flex">
    <Container component="main" className={classes.main} maxWidth="md">
      <CssBaseline />
      <Header />
      <main className={classes.mainWrapper}>
        <Paper
          component="section"
          elevation={3}
          className={classes.sectionWrapper}
        >
          <Box p={2}>
            <InputEditor input={input} setInput={setInput} />
          </Box>
        </Paper>
        <Paper
          component="section"
          elevation={1}
          className={classes.sectionWrapper}
        >
          <Box p={1}>
            <OptionsControl
              ipKind={ipKind}
              toggleIpv4={toggleIpv4}
              toggleIpv6={toggleIpv6}
              handleAggregate={handleAggregate}
              ref={controlRef}
            />
          </Box>
        </Paper>
        <Paper
          component="section"
          elevation={3}
          className={classes.sectionWrapper}
        >
          <Box p={2}>
            <OutputEditor ipKind={ipKind} output={output} />
          </Box>
        </Paper>
      </main>
      <Footer />
    </Container>
  );
}

export default App;
