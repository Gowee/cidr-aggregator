import React, { useState, useRef, useEffect } from "react";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";

import Header from "./components/Header";
import Footer from "./components/Footer";
import InputEditor from "./components/InputEditor";
import OutputEditor from "./components/OutputEditor";
import OptionsControl from "./components/OptionsControl";

function App() {
  const controlRef = useRef(null as any);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState(undefined as any);
  const [ipKind, setIpKind] = useState("both");
  const [bogonFilter, setBogonFilter] = useState(
    undefined as "reserved" | undefined
  );
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
  const toggleReservedFilter = () => {
    setBogonFilter((prev) => {
      if (bogonFilter === "reserved") {
        return undefined;
      } else {
        return "reserved";
      }
    });
  };
  const handleAggregate = async (reverse = false) => {
    const { aggregate } = await import("../../pkg/cidr_aggregator.js");
    setOutput(
      Object.assign(
        { reverse },
        await aggregate(input, reverse, bogonFilter === "reserved")
      )
    );
    controlRef?.current &&
      controlRef.current.scrollIntoView({ behavior: "smooth" });
  };
  useEffect(() => {
    output && handleAggregate(output.reverse);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bogonFilter]);

  return (
    <Container component="main" sx={{ mt: 5, mb: 2 }} maxWidth="md">
      <CssBaseline />
      <Header />
      <main>
        <Paper
          component="section"
          elevation={3}
          sx={{ mt: 1, mb: 1 }}
        >
          <Box sx={{ p: 2 }}>
            <InputEditor input={input} setInput={setInput} />
          </Box>
        </Paper>
        <Paper
          component="section"
          elevation={1}
          sx={{ mt: -1, mb: -1, p: 1 }}
        >
          <OptionsControl
            ipKind={ipKind}
            toggleIpv4={toggleIpv4}
            toggleIpv6={toggleIpv6}
            bogonFilter={bogonFilter}
            toggleReservedFilter={toggleReservedFilter}
            handleAggregate={handleAggregate}
            ref={controlRef}
          />
        </Paper>
        <Paper
          component="section"
          elevation={3}
          sx={{ mt: 1, mb: 1 }}
        >
          <Box sx={{ p: 2 }}>
            <OutputEditor ipKind={ipKind} output={output} />
          </Box>
        </Paper>
      </main>
      <Footer />
    </Container>
  );
}

export default App;
