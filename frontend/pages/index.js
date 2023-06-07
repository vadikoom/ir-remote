import { useEffect } from "react";
import { api } from "../lib/api";
import { commands } from "../lib/commands";
import { StatusBage } from "../components/StatusBage";

const HomePage = () => {
  useEffect(() => {
    window._api = api;
    window._commands = commands;
  }, []);

  return (
    <>
      <StatusBage />
      <br />
      <br />
      <button onClick={() => commands.on22Cooling()}>On 22 Cooling</button>
      <br />
      <br />
      <button onClick={() => commands.off()}>Off</button>
      <br />
      <br />
    </>
  );
};

export default HomePage;
