import {
  ConfigSeed,
  InstallAgentsHapps,
  Orchestrator,
} from "@holochain/tryorama";
export type FunctionType = (
  orchestrator: Orchestrator<unknown>,
  config: ConfigSeed,
  installable: InstallAgentsHapps
) => any;
