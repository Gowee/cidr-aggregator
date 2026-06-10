import React from "react";
import { render, screen } from "@testing-library/react";
import App from "./App";

test("renders CIDR Aggregator heading", () => {
  render(<App />);
  const heading = screen.getByText(/CIDR Aggregator/i);
  expect(heading).toBeInTheDocument();
});
